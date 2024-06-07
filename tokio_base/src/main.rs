#[warn(dead_code)]
use std::ops::Sub;

use clap::Parser;
use tokio::{sync::mpsc, time};

#[derive(Parser, Clone, Debug, Default)]
// 由clap库提供的属性宏。用于配置结构体，使其能够从命令行参数中解析值
// name选项指定这个命令行工具的名称
#[clap(name = "example")] 
struct Opts{
    /// 每个字段都有对应的clap属性
    // 事件的大小
    #[clap(long, short, default_value="16")]
    size : i64,

    // 事件类型
    #[clap(long, short='t', default_value="0")]
    etype:i64,

    // 工作线程的数量
    #[clap(long, short, default_value="100")]
    worker:usize,

    // 每个工作线程的事件数量
    #[clap(long, short, default_value="100")]
    event:usize,

    // 工作线程队列的大小
    #[clap(long, short, default_value="16")]
    queue: usize,

    // 输出为csv格式,默认为JSON
    #[clap(long, short)]
    csv: bool,

    // CPU性能分析
    #[clap(long, default_value="")]
    cpuprofile:String,

    // 更多的输出信息
    #[clap(long, short)]
    verbose:bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Event {
    Int(i64),
    Float(f64),
    Str(String),
    StaticStr(&'static str),
}


impl Event {
    // 新建事件
    pub fn new(etype: i64, seed: i64) -> Self{
        match etype{
            0 => Self::Int(seed), //根据传入参数创建指定整数
            1 => Self::Str("A".repeat(seed as usize)),//把字母A重复指定次数
            2 => {
                let s = Box::new("A".repeat(seed as usize));
                let s: &'static str = Box::leak(s).as_str();//内存泄露，扩大其生命周期
                Event::StaticStr(s)
            },
            3 =>  Self::Str("A".repeat(seed as usize)),
            4 => Self::Float(seed as f64 / 100.0),
            _ => panic!("无效的事件类型"),
        }
    }

    // 检查事件是否是退出事件
    // 目前的检查逻辑就是看传入的是不是退出码-1或者是退出字符串exit
    pub fn is_exit(&self) -> bool{
        match self{
            Event::Int(v) => (-1).eq(v),
            Event::Str(v) => v.eq("exit"),
            Event::StaticStr(v) => "exit".eq(*v),
            Event::Float(v) => {
                v + 1.0 <= 1e-6
            }
        }
    }
}

// 消息发送和接收 基于tokio的实现
type EventSender = mpsc::Sender<Event>;
type EventReceiver = mpsc::Receiver<Event>;

// 工作线程函数
async fn worker(mut queue: EventReceiver, mut done: mpsc::Sender<usize>, events: usize){
    let mut n = 0;
    // 对于每个特定的协程 接收其协程队列里面的所有消息
    // 除了退出消息之外的所有消息都会计数
    for _i in 0..events{
        if let Some(event) = queue.recv().await {
            if !event.is_exit() {
                n += 1;
            }
        }
    }
    // 最后给对方返回本次异步接收到的消息数目
    done.send(n).await.unwrap();
}

// 分发事件到指定的地址
async fn dispatch_to(event:Event,events: usize,addr :EventSender){
    for _ in 0..events{
        addr.send(event.clone()).await.unwrap()
    }
    addr.closed().await;
}

// 分发器函数
async fn dispatch(opts: Opts, address: Vec<EventSender>) {
    let event = Event::new(opts.etype, opts.size);
    address.into_iter().for_each(|addr| {
        let event =  event.clone();
        let events = opts.event;
        tokio::spawn(async move {
            dispatch_to(event, events, addr).await
        });
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let opts = Opts::parse();

    let (done_tx,mut done_rx) = mpsc::channel(opts.worker);
    let total_events = opts.event * opts.worker;

    // 启动工作线程
    // address是所有工作线程的发送端口
    let mut address = vec![];
    for _ in 0..opts.worker{
        let (tx, rx) = mpsc::channel(opts.queue);
        address.push(tx);
        tokio::spawn(worker(rx, done_tx.clone(), opts.event));
    }
    // 记录启动时间
    let t1 = time::Instant::now();

    // 启动分发器
    tokio::spawn(dispatch(opts.clone(), address));

    // 等待所有任务完成
    let mut sn: usize = 0;
    for _i in 0..opts.worker {
        if let Some(n) = done_rx.recv().await {
            sn += n;
        }
    }
    let t2 = time::Instant::now();
    if opts.verbose{
        println!("所有的事件{},一共花费了{}的时间",total_events,sn);
    }

    // t2和t1做差 计算所花费的时间
    let ts = t2.sub(t1).as_secs_f64();
    // 计算速度
    let speed = (total_events) as f64 / ts;
    let etype = match opts.etype {
        0 => "int",
        1 => "str",
        2 => "str_ptr",
        3 => "str_clone",
        _ => "unknown",
    };

    if opts.csv {
        println!("rust,{},{},{},{:.3},{:.0}", etype, opts.worker, opts.event, ts, speed)
    }else{
        println!("workers   : {}", opts.worker);
        println!("events    : {}", opts.event);
        println!("time used : {:.3}s", ts);
        println!("Speed     : {:.0}/s", speed);
    }
    Ok(())
}
#可执行文件的路径
TOKIO_BASE = tokio_base/target/release/tokio_base

#默认参数定义
worker = 1000    #工作线程的数量
event = 1000     #每个工作线程的事件数量
queue = 256      # 工作线程队列的大小
etype = 0        # 事件类型
esize = 64       # 事件大小
verbose =        # 是否启用详细输出

#解析命令行参数

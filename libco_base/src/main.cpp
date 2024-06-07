#include <iostream>
#include <vector>
#include <queue>
#include <thread>
#include <atomic>
#include <chrono>
#include <string>
#include "libco/co_routine.h"
#include "cxxopts/include/cxxopts.hpp"


enum class EventType {
    INT,
    STRING,
    STRING_PTR,
    STRING_CLONE,
    FLOAT,
};

struct Event {
    EventType type;
    union {
        int64_t int_val;
        double float_val;
    };
    std::string str_val;
    const char* str_ptr_val;

    Event() : type(EventType::INT), int_val(0) {}
};

struct WorkerContext {
    std::queue<Event> queue;
    std::atomic<bool> done{false};
};

void worker_routine(void* arg) {
    WorkerContext* ctx = static_cast<WorkerContext*>(arg);
    size_t count = 0;
    while (!ctx->done) {
        if (!ctx->queue.empty()) {
            Event event = std::move(ctx->queue.front());
            ctx->queue.pop();
            if (event.type == EventType::INT && event.int_val == -1) {
                break;
            }
            ++count;
        }
        co_sleep(1);
    }
    std::cout << "Processed " << count << " events." << std::endl;
}

void sender_routine(void* arg) {
    WorkerContext* ctx = static_cast<WorkerContext*>(arg);
    for (int i = 0; i < 100; ++i) {
        Event event;
        event.type = EventType::INT;
        event.int_val = i;
        ctx->queue.push(event);
        co_sleep(1);
    }
    ctx->done = true;
}

int main(int argc, char** argv) {
    cxxopts::Options options("PerformanceTest", "Performance testing with coroutines");
    options.add_options()
        ("s,size", "Event size", cxxopts::value<int>()->default_value("16"))
        ("t,etype", "Event type", cxxopts::value<int>()->default_value("0"))
        ("w,worker", "Number of workers", cxxopts::value<int>()->default_value("100"))
        ("e,event", "Number of events per worker", cxxopts::value<int>()->default_value("100"))
        ("q,queue", "Queue size", cxxopts::value<int>()->default_value("16"))
        ("c,csv", "Output in CSV format", cxxopts::value<bool>()->default_value("false"))
        ("v,verbose", "Verbose output", cxxopts::value<bool>()->default_value("false"));
    
    auto result = options.parse(argc, argv);

    int num_workers = result["worker"].as<int>();
    int num_events = result["event"].as<int>();

    std::vector<stCoRoutine_t*> workers;
    std::vector<WorkerContext> contexts(num_workers);

    for (int i = 0; i < num_workers; ++i) {
        stCoRoutine_t* worker;
        co_create(&worker, nullptr, worker_routine, &contexts[i]);
        workers.push_back(worker);
    }

    auto start_time = std::chrono::high_resolution_clock::now();

    for (int i = 0; i < num_workers; ++i) {
        co_resume(workers[i]);
    }

    std::vector<stCoRoutine_t*> senders;
    for (int i = 0; i < num_workers; ++i) {
        stCoRoutine_t* sender;
        co_create(&sender, nullptr, sender_routine, &contexts[i]);
        senders.push_back(sender);
    }

    for (int i = 0; i < num_workers; ++i) {
        co_resume(senders[i]);
    }

    for (int i = 0; i < num_workers; ++i) {
        co_eventloop(co_get_epoll_ct(), nullptr, nullptr);
    }

    auto end_time = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> duration = end_time - start_time;

    std::cout << "Total time: " << duration.count() << " seconds" << std::endl;

    return 0;
}

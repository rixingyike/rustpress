---
title: "Go语言并发模式深度解析"
createTime: 2024-03-15 14:20:00
tags: ["Go", "并发编程", "goroutine", "channel"]
---

# Go语言并发模式深度解析

Go语言以其出色的并发编程能力而闻名，goroutine和channel是其并发模型的核心。本文将深入探讨Go语言的并发模式和实践技巧。

## 并发基础

### goroutine
轻量级线程，由Go运行时管理。

```go
package main

import (
    "fmt"
    "time"
)

func sayHello() {
    for i := 0; i < 5; i++ {
        fmt.Println("Hello")
        time.Sleep(100 * time.Millisecond)
    }
}

func sayWorld() {
    for i := 0; i < 5; i++ {
        fmt.Println("World")
        time.Sleep(100 * time.Millisecond)
    }
}

func main() {
    go sayHello()
    go sayWorld()
    
    // 等待goroutine执行完成
    time.Sleep(1 * time.Second)
}
```

### channel
goroutine之间的通信机制。

```go
package main

import "fmt"

func sum(numbers []int, result chan int) {
    total := 0
    for _, num := range numbers {
        total += num
    }
    result <- total // 发送结果到channel
}

func main() {
    numbers := []int{1, 2, 3, 4, 5}
    result := make(chan int)
    
    go sum(numbers[:len(numbers)/2], result)
    go sum(numbers[len(numbers)/2:], result)
    
    // 接收结果
    part1, part2 := <-result, <-result
    fmt.Printf("总和: %d\n", part1+part2)
}
```

## 并发模式

### 1. Worker Pool模式
使用固定数量的worker处理任务。

```go
package main

import (
    "fmt"
    "sync"
    "time"
)

func worker(id int, jobs <-chan int, results chan<- int) {
    for job := range jobs {
        fmt.Printf("Worker %d 开始处理任务 %d\n", id, job)
        time.Sleep(time.Second) // 模拟工作
        results <- job * 2
        fmt.Printf("Worker %d 完成任务 %d\n", id, job)
    }
}

func main() {
    const numJobs = 5
    jobs := make(chan int, numJobs)
    results := make(chan int, numJobs)
    
    // 启动3个worker
    for w := 1; w <= 3; w++ {
        go worker(w, jobs, results)
    }
    
    // 发送任务
    for j := 1; j <= numJobs; j++ {
        jobs <- j
    }
    close(jobs)
    
    // 收集结果
    for a := 1; a <= numJobs; a++ {
        <-results
    }
}
```

### 2. Fan-out/Fan-in模式
多个goroutine处理输入，然后合并结果。

```go
package main

import (
    "fmt"
    "sync"
)

func producer(nums []int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for _, n := range nums {
            out <- n
        }
    }()
    return out
}

func square(in <-chan int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for n := range in {
            out <- n * n
        }
    }()
    return out
}

func merge(channels ...<-chan int) <-chan int {
    var wg sync.WaitGroup
    out := make(chan int)
    
    // 启动goroutine从每个channel读取
    output := func(c <-chan int) {
        defer wg.Done()
        for n := range c {
            out <- n
        }
    }
    
    wg.Add(len(channels))
    for _, c := range channels {
        go output(c)
    }
    
    // 等待所有goroutine完成
    go func() {
        wg.Wait()
        close(out)
    }()
    
    return out
}

func main() {
    nums := []int{1, 2, 3, 4, 5}
    
    // 生成数据
    in := producer(nums)
    
    // 并行处理
    c1 := square(in)
    c2 := square(in)
    
    // 合并结果
    for n := range merge(c1, c2) {
        fmt.Println(n)
    }
}
```

### 3. Pipeline模式
将处理过程分解为多个阶段。

```go
package main

import "fmt"

func generator(nums ...int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for _, n := range nums {
            out <- n
        }
    }()
    return out
}

func square(in <-chan int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for n := range in {
            out <- n * n
        }
    }()
    return out
}

func main() {
    // 创建pipeline: generator -> square
    for n := range square(generator(1, 2, 3, 4, 5)) {
        fmt.Println(n)
    }
}
```

## 同步原语

### sync.WaitGroup
等待一组goroutine完成。

```go
package main

import (
    "fmt"
    "sync"
    "time"
)

func worker(id int, wg *sync.WaitGroup) {
    defer wg.Done()
    fmt.Printf("Worker %d 开始工作\n", id)
    time.Sleep(time.Second)
    fmt.Printf("Worker %d 完成工作\n", id)
}

func main() {
    var wg sync.WaitGroup
    
    for i := 1; i <= 5; i++ {
        wg.Add(1)
        go worker(i, &wg)
    }
    
    wg.Wait()
    fmt.Println("所有工作完成")
}
```

### sync.Mutex
保护共享资源的互斥锁。

```go
package main

import (
    "fmt"
    "sync"
)

type Counter struct {
    mu    sync.Mutex
    value int
}

func (c *Counter) Increment() {
    c.mu.Lock()
    defer c.mu.Unlock()
    c.value++
}

func (c *Counter) Value() int {
    c.mu.Lock()
    defer c.mu.Unlock()
    return c.value
}

func main() {
    var wg sync.WaitGroup
    counter := &Counter{}
    
    for i := 0; i < 1000; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            counter.Increment()
        }()
    }
    
    wg.Wait()
    fmt.Printf("最终计数: %d\n", counter.Value())
}
```

## 最佳实践

### 1. 避免goroutine泄漏
确保goroutine能够正常退出。

### 2. 使用context进行取消和超时控制

```go
package main

import (
    "context"
    "fmt"
    "time"
)

func worker(ctx context.Context) {
    for {
        select {
        case <-ctx.Done():
            fmt.Println("收到取消信号，退出")
            return
        default:
            // 正常工作
            fmt.Println("工作中...")
            time.Sleep(500 * time.Millisecond)
        }
    }
}

func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
    defer cancel()
    
    go worker(ctx)
    
    time.Sleep(3 * time.Second)
}
```

### 3. 合理使用缓冲channel

```go
// 无缓冲channel
ch1 := make(chan int)

// 有缓冲channel
ch2 := make(chan int, 10)
```

### 4. 使用select处理多个channel

```go
package main

import (
    "fmt"
    "time"
)

func main() {
    ch1 := make(chan string)
    ch2 := make(chan string)
    
    go func() {
        time.Sleep(1 * time.Second)
        ch1 <- "来自ch1"
    }()
    
    go func() {
        time.Sleep(2 * time.Second)
        ch2 <- "来自ch2"
    }()
    
    for i := 0; i < 2; i++ {
        select {
        case msg1 := <-ch1:
            fmt.Println(msg1)
        case msg2 := <-ch2:
            fmt.Println(msg2)
        }
    }
}
```

## 性能考虑

### 1. goroutine开销
每个goroutine初始栈大小为2KB，远小于线程。

### 2. channel性能
无缓冲channel比有缓冲channel更快，但可能导致阻塞。

### 3. 避免过度并发
过多的goroutine可能导致调度开销增加。

## 调试和监控

### 1. 使用pprof分析性能
```bash
go tool pprof http://localhost:6060/debug/pprof/goroutine
```

### 2. 使用trace工具分析执行
```bash
go tool trace trace.out
```

## 总结

Go语言的并发模型简单而强大，通过goroutine和channel的组合，可以构建高效的并发程序。掌握这些并发模式对于编写高性能的Go应用程序至关重要。
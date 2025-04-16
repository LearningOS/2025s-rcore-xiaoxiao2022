## 实验题：

实现了新的系统调用`sys_spawn`和`stride`调度算法



## 问答题：

1. 关于第一个问题：
   当 p1.stride = 255, p2.stride = 250 时，p2执行后 stride 变为 250 + 10 = 260，但用8位无符号整数存储会溢出变为 260 - 256 = 4。
   此时比较 255 和 4，理论上应该选择较小的4(p2)，但实际上应该轮到p1执行。这是因为溢出导致比较失效。
2. 关于 STRIDE_MAX - STRIDE_MIN <= BigStride / 2 的说明：
   当所有优先级 >= 2 时，两个进程的 stride 差值每次最多增加 |pass1 - pass2|。由于 pass >= 2，最大差值为 BigStride/2（因为最小步长是2，最大是...）。这样能保证即使一个刚更新，另一个即将更新，差值也不会超过 BigStride/2。
3. 比较器实现思路：
   利用溢出特性，我们可以认为当两个 stride 的差值的绝对值 > BigStride/2 时，较大的数实际更小（因为发生了溢出）。

补全代码：

```
rustCopy Codeuse core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const BIG_STRIDE: u64 = 255;
        const HALF_STRIDE: u64 = BIG_STRIDE / 2;
        
        if self.0 == other.0 {
            return None;
        }
        
        let diff = if self.0 > other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        };
        
        if diff <= HALF_STRIDE {
            self.0.partial_cmp(&other.0)
        } else {
            // 发生溢出，实际大小关系反转
            other.0.partial_cmp(&self.0)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

这个比较器的核心逻辑是：

1. 计算两个 stride 的绝对差值
2. 如果差值 <= BigStride/2，正常比较
3. 如果差值 > BigStride/2，说明发生了溢出，实际大小关系应该反转



## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 交流群的同学和助教

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > rCore-Camp-Guide-2025S文档

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
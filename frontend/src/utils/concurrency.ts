/** 以固定并发数执行一组异步任务，全部完成后返回 */
export async function runTasksWithConcurrency(
  tasks: Array<() => Promise<void>>,
  concurrency: number = 6,
) {
  let cursor = 0
  const runNext = async (): Promise<void> => {
    while (cursor < tasks.length) {
      const taskIndex = cursor++
      await tasks[taskIndex]()
    }
  }
  await Promise.all(Array.from(
    { length: Math.min(concurrency, tasks.length) },
    () => runNext(),
  ))
}

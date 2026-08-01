export function proxyNodeDeleteSuccessMessage(
  clearedSystemProxy: boolean,
  clearedExternalModelsProxy: boolean,
): string {
  if (clearedSystemProxy && clearedExternalModelsProxy) {
    return '代理节点已删除，系统默认代理和外部模型目录代理已自动清除'
  }
  if (clearedSystemProxy) {
    return '代理节点已删除，系统默认代理已自动清除'
  }
  if (clearedExternalModelsProxy) {
    return '代理节点已删除，外部模型目录代理已自动清除'
  }
  return '代理节点已删除'
}

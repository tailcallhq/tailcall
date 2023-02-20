package tailcall.gateway.lambda

final case class CompilationContext(level: Int, index: Int) {
  def withNextLevel: CompilationContext =
    CompilationContext(level + 1, index = 0)

  def withNextIndex: CompilationContext = CompilationContext(level, index + 1)
}

object CompilationContext {
  def initial: CompilationContext = CompilationContext(0, 0)
}

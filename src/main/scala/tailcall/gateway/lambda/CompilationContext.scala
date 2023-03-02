package tailcall.gateway.lambda

final case class CompilationContext(level: Int):
  def next: CompilationContext = CompilationContext(level + 1)

object CompilationContext:
  def initial: CompilationContext = CompilationContext(0)

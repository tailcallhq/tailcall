package tailcall.gateway.remote

import zio.schema.Schema

sealed trait Lambda[-A, +B] {
  def compile(context: CompilationContext): DynamicEval
  final def toFunction: Remote[A] => Remote[B] = ???
}

object Lambda {
  def fromFunction[A, B](f: Remote[A] => Remote[B]): A ~> B = ???

  object unsafe {
    object attempt {
      def apply[A, B](cmp: CompilationContext => DynamicEval): Lambda[A, B] =
        new ~>[A, B] {
          override def compile(context: CompilationContext): DynamicEval =
            cmp(context)
        }
    }
  }

  implicit val anySchema: Schema[Lambda[_, _]] = Schema[DynamicEval].transform(
    eval => Lambda.unsafe.attempt(_ => eval),
    _.compile(CompilationContext.initial)
  )

  implicit def schema[A, B]: Schema[A ~> B] =
    anySchema.asInstanceOf[Schema[A ~> B]]
}

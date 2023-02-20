package tailcall.gateway.remote

/**
 * Remote[A] Allows for any arbitrary computation that can
 * be serialized and when evaluated produces a result of
 * type A. This is the lowest level primitive that’s
 * extremely powerful. We use this inside the compiler to
 * convert the composition logic into some form of a Remote.
 */

//object Lambda
//    extends RemoteCtors
//    with StringOps
//    with SeqOps
//    with BooleanOps
//    with EitherOps
//    with OptionOps
//    with ContextOps
//    with DynamicValueOps
//    with TupleOps
//    with MapOps
//    with RemoteOps {
//
//  object unsafe {
//    object attempt {
//      def apply[A](eval: CompilationContext => DynamicEval): Remote[A] =
//        Lambda.unsafe.attempt(eval)
//    }
//  }
//
//  implicit val anySchema: Schema[Remote[_]] = Schema[DynamicEval].transform(
//    eval => unsafe.attempt(_ => eval),
//    _.compile(CompilationContext.initial)
//  )
//
//  implicit def schema[A]: Schema[Remote[A]] =
//    anySchema.asInstanceOf[Schema[Remote[A]]]
//
//  implicit def remoteFunctionSchema[A, B]: Schema[Remote[A] => Remote[B]] =
//    Schema[A ~> B].transform(_.toFunction, Lambda.fromFunction)
//}

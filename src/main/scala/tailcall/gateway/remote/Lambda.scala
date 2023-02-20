package tailcall.gateway.remote

import tailcall.gateway.ast.Endpoint
import tailcall.gateway.remote.DynamicEval._
import zio.schema.{DynamicValue, Schema}
import zio.{Chunk, ZIO}

sealed trait Lambda[-A, +B] {
  self =>
  final def apply[A1 <: A](a: A1)(implicit ev: Constructor[A1]): Remote[B] =
    Lambda(a) >>> self

  final def apply(a: Remote[A]): Remote[B] = a >>> self

  final def >>>[B1 >: B, C](other: B1 ~> C): A ~> C =
    Lambda
      .unsafe
      .attempt[A, C](ctx =>
        DynamicEval.FunctionOperations(
          DynamicEval.FunctionOperations.Pipe(compile(ctx), other.compile(ctx))
        )
      )

  final def pipe[B1 >: B, C](other: B1 ~> C): A ~> C = self >>> other

  final def <<<[A1 <: A, C](other: C ~> A1): C ~> B = other >>> self

  def compile(ctx: CompilationContext): DynamicEval

  final def evaluateWith(a: A): ZIO[LambdaRuntime, Throwable, B] = evaluate(a)

  final def evaluate: LExit[LambdaRuntime, Throwable, A, B] =
    LambdaRuntime.evaluate(self)

  final def toFunction: Remote[A] => Remote[B] = a => a >>> self

  def =:=[A1 <: A, B1 >: B](
    other: A1 ~> B1
  )(implicit tag: Equatable[B1]): A1 ~> Boolean =
    Lambda
      .unsafe
      .attempt(ctx => EqualTo(self.compile(ctx), other.compile(ctx), tag.any))

  def debug(message: String): A ~> B =
    Lambda.unsafe.attempt(ctx => DynamicEval.Debug(self.compile(ctx), message))
}

object Lambda {

  def apply[B](b: B)(implicit ctor: Constructor[B]): Any ~> B =
    Lambda
      .unsafe
      .attempt(_ =>
        DynamicEval.FunctionOperations(
          DynamicEval
            .FunctionOperations
            .Literal(ctor.schema.toDynamic(b), ctor.any)
        )
      )

  def batch[A, B, C](
    from: Remote[Seq[A]],
    to: Remote[Seq[B]] => Remote[Seq[C]],
    ab: Remote[A] => Remote[B],
    ba: Remote[B] => Remote[A],
    cb: Remote[C] => Remote[B]
  )(implicit
    ctorB: Constructor[B],
    ctorC: Constructor[C],
    ctorF: Constructor[(A, Option[C])]
  ) = {

    val v = from.map(ab(_))
    v.map(i =>
      fromTuple(
        ba(i),
        to(v)
          .map(c => fromTuple((cb(c), c)))
          .groupBy(_._1)
          .get(i)
          .flatMap(x => x.map(_._2).head) // Todo: Add flatten in Option
      )
    )
  }

  def fromTuple[A1, A2](t: (Remote[A1], Remote[A2])): Remote[(A1, A2)] =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval.TupleOperations(
          TupleOperations.Cons(Chunk(t._1.compile(ctx), t._2.compile(ctx)))
        )
      )

  def die(msg: String): Remote[Nothing] = die(Lambda(msg))

  def die(msg: Remote[String]): Remote[Nothing] =
    Lambda.unsafe.attempt(ctx => DynamicEval.Die(msg.compile(ctx)))

  def dynamicValue[A](a: A)(implicit schema: Schema[A]): Remote[DynamicValue] =
    Lambda(Schema.toDynamic(a))

  def flatten[A, B](ab: A ~> (A ~> B)): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval.FunctionOperations(
          DynamicEval.FunctionOperations.Flatten(ab.compile(ctx))
        )
      )

  def fromEither[E, A](a: Either[Remote[E], Remote[A]]): Remote[Either[E, A]] =
    Lambda
      .unsafe
      .attempt(ctx =>
        EitherOperations(EitherOperations.Cons(a match {
          case Left(value)  => Left(value.compile(ctx))
          case Right(value) => Right(value.compile(ctx))
        }))
      )

  def fromEndpoint(endpoint: Endpoint): DynamicValue ~> DynamicValue =
    Lambda.fromFunction[DynamicValue, DynamicValue](input =>
      Lambda
        .unsafe
        .attempt(ctx => DynamicEval.EndpointCall(endpoint, input.compile(ctx)))
    )

  def fromFunction[A, B](f: Remote[A] => Remote[B]): A ~> B =
    Lambda
      .unsafe
      .attempt[A, B] { ctx =>
        val next = ctx.withNextLevel

        val key  = EvaluationContext.Key.fromContext(next)
        val body = f(
          Lambda
            .unsafe
            .attempt(_ =>
              DynamicEval
                .FunctionOperations(DynamicEval.FunctionOperations.Lookup(key))
            )
        ).compile(next)

        DynamicEval.FunctionOperations(
          DynamicEval.FunctionOperations.FunctionDefinition(key, body)
        )
      }

  def fromMap[A, B](a: Map[Remote[A], Remote[B]]): Remote[Map[A, B]] =
    Lambda
      .unsafe
      .attempt(ctx =>
        MapOperations(MapOperations.Cons(Chunk.fromIterable(a.map {
          case (k, v) => k.compile(ctx) -> v.compile(ctx)
        })))
      )

  def fromOption[A](a: Option[Remote[A]]): Remote[Option[A]] =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval
          .OptionOperations(OptionOperations.Cons(a.map(_.compile(ctx))))
      )

  def fromSeq[A](
    a: Seq[Remote[A]]
  )(implicit ctor: Constructor[A]): Remote[Seq[A]] =
    Lambda
      .unsafe
      .attempt(ctx =>
        SeqOperations(SeqOperations.Sequence(
          Chunk.fromIterable(a.map(_.compile(ctx))),
          ctor.asInstanceOf[Constructor[Any]]
        ))
      )

  def fromTuple[A1, A2, A3](
    t: (Remote[A1], Remote[A2], Remote[A3])
  ): Remote[(A1, A2, A3)] =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval.TupleOperations(TupleOperations.Cons(
          Chunk(t._1.compile(ctx), t._2.compile(ctx), t._3.compile(ctx))
        ))
      )

  def fromTuple[A1, A2, A3, A4](
    t: (Remote[A1], Remote[A2], Remote[A3], Remote[A4])
  ): Remote[(A1, A2, A3, A4)] =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval.TupleOperations(TupleOperations.Cons(Chunk(
          t._1.compile(ctx),
          t._2.compile(ctx),
          t._3.compile(ctx),
          t._4.compile(ctx)
        )))
      )

  def none[B]: Remote[Option[B]] =
    Lambda.unsafe.attempt(_ => OptionOperations(OptionOperations.Cons(None)))

  def record(fields: (String, Remote[DynamicValue])*): Remote[DynamicValue] =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval.Record(Chunk.fromIterable(fields.map { case (k, v) =>
          k -> v.compile(ctx)
        }))
      )

  object unsafe {
    def attempt[A, B](c: CompilationContext => DynamicEval): Lambda[A, B] =
      new Lambda[A, B] {
        override def compile(ctx: CompilationContext): DynamicEval = c(ctx)
      }
  }

  implicit val anySchema: Schema[Lambda[_, _]] = Schema[DynamicEval].transform(
    exe => Lambda.unsafe.attempt(_ => exe),
    _.compile(CompilationContext.initial)
  )

  implicit def schema[A, B]: Schema[A ~> B] =
    anySchema.asInstanceOf[Schema[A ~> B]]

}

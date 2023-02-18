package tailcall.gateway.remote

import tailcall.gateway.ast.Endpoint
import zio.Chunk
import zio.schema.{DynamicValue, Schema}

trait RemoteCtors {
  def apply[A](a: A)(implicit ctor: Constructor[A]): Remote[A] =
    Remote.unsafe.attempt(_ => DynamicEval.cons(ctor.schema.toDynamic(a), ctor))

  def fromFunction[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] =
    Remote
      .unsafe
      .attempt { ctx =>
        val next = ctx.withNextLevel
        val id   = DynamicEval.lookup(next)
        DynamicEval
          .functionDef(id, ab(Remote.unsafe.attempt[A](_ => id)).compile(next))
      }

  def fromSeq[A](a: Seq[Remote[A]]): Remote[Seq[A]] =
    Remote.unsafe.attempt(ctx => DynamicEval.seq(a.map(_.compile(ctx))))

  def fromMap[A, B](a: Map[Remote[A], Remote[B]]): Remote[Map[A, B]] =
    Remote
      .unsafe
      .attempt(ctx =>
        DynamicEval.map(a.map { case (k, v) =>
          k.compile(ctx) -> v.compile(ctx)
        })
      )

  def fromEither[E, A](a: Either[Remote[E], Remote[A]]): Remote[Either[E, A]] =
    Remote
      .unsafe
      .attempt(ctx =>
        DynamicEval.either(a match {
          case Left(value)  => Left(value.compile(ctx))
          case Right(value) => Right(value.compile(ctx))
        })
      )

  def fromOption[A](a: Option[Remote[A]]): Remote[Option[A]] =
    Remote.unsafe.attempt(ctx => DynamicEval.option(a.map(_.compile(ctx))))

  def none[B]: Remote[Option[B]] = Remote.unsafe.attempt(_ => DynamicEval.none)

  def fromEndpoint(endpoint: Endpoint): Remote[DynamicValue => DynamicValue] =
    Remote.fromFunction[DynamicValue, DynamicValue](input =>
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.endpoint(endpoint, input.compile(ctx)))
    )

  def dynamicValue[A](a: A)(implicit schema: Schema[A]): Remote[DynamicValue] =
    Remote(Schema.toDynamic(a))

  def record(fields: (String, Remote[DynamicValue])*): Remote[DynamicValue] =
    Remote
      .unsafe
      .attempt(ctx =>
        DynamicEval.record(fields.map { case (k, v) => k -> v.compile(ctx) })
      )

  def die(msg: Remote[String]): Remote[Nothing] =
    Remote.unsafe.attempt(ctx => DynamicEval.die(msg.compile(ctx)))

  def die(msg: String): Remote[Nothing] = die(Remote(msg))

  def fromTuple[A1, A2](t: (Remote[A1], Remote[A2])): Remote[(A1, A2)] =
    Remote
      .unsafe
      .attempt(ctx =>
        DynamicEval.tuple(Chunk(t._1.compile(ctx), t._2.compile(ctx)))
      )

  def fromTuple[A1, A2, A3](
    t: (Remote[A1], Remote[A2], Remote[A3])
  ): Remote[(A1, A2, A3)] =
    Remote
      .unsafe
      .attempt(ctx =>
        DynamicEval
          .tuple(Chunk(t._1.compile(ctx), t._2.compile(ctx), t._3.compile(ctx)))
      )

  def fromTuple[A1, A2, A3, A4](
    t: (Remote[A1], Remote[A2], Remote[A3], Remote[A4])
  ): Remote[(A1, A2, A3, A4)] =
    Remote
      .unsafe
      .attempt(ctx =>
        DynamicEval.tuple(Chunk(
          t._1.compile(ctx),
          t._2.compile(ctx),
          t._3.compile(ctx),
          t._4.compile(ctx)
        ))
      )

  def batch[A, B, C](
    from: Remote[Seq[A]],
    to: Remote[Seq[B]] => Remote[Seq[C]],
    ab: Remote[A] => Remote[B],
    ba: Remote[B] => Remote[A],
    cb: Remote[C] => Remote[B]
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

  def flatten[A](r: Remote[Remote[A]]): Remote[A] =
    Remote.unsafe.attempt(ctx => DynamicEval.flatten(r.compile(ctx)))

  def recurse[A, B](f: Remote[(A, A => B)] => Remote[B]): Remote[A => B] =
    Remote
      .unsafe
      .attempt(ctx => DynamicEval.recurse(Remote.fromFunction(f).compile(ctx)))
}

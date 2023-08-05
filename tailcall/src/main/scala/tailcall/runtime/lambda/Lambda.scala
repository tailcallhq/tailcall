package tailcall.runtime.lambda

import tailcall.runtime.JsonT
import tailcall.runtime.lambda.Expression._
import tailcall.runtime.model.Endpoint
import tailcall.runtime.service.EvaluationContext.Binding
import zio.UIO
import zio.json.JsonCodec
import zio.schema.codec.JsonCodec.jsonCodec
import zio.schema.{DynamicValue, Schema}

sealed trait Lambda[-A, +B] {
  self =>
  final def >>>[C](other: B ~> C): A ~> C = Lambda.unsafe.attempt(ctx => Pipe(self.compile(ctx), other.compile(ctx)))

  final def apply[A2, A1 <: A](r: A2 ~> A1): A2 ~> B = r >>> self

  def compile(context: CompilationContext): Expression

  final def compile: Expression = compile(CompilationContext.initial)

  final def toDynamic[B1 >: B](implicit ev: Schema[B1]): A ~> DynamicValue = self >>> Lambda.dynamic.toDynamic[B1]

  final def widen[B1](implicit ev: B <:< B1): A ~> B1 = self.asInstanceOf[A ~> B1]

  /**
   * NOTE: Only used for testing purposes. The operator is
   * ignore from the expression tree at the time of
   * serialization.
   */
  final private[tailcall] def tap(f: B => UIO[Unit]): A ~> B =
    Lambda.unsafe.attempt(_ => Unsafe(Unsafe.Tap(self.compile, f.asInstanceOf[Any => UIO[Unit]])))
}

object Lambda {
  implicit def json[A, B]: JsonCodec[A ~> B] = jsonCodec[A ~> B](schema)

  implicit def schema[A, B]: Schema[A ~> B] = anySchema.asInstanceOf[Schema[A ~> B]]

  implicit def schemaFunction[A, B]: Schema[A ~>> B] =
    Schema[A ~> B].transform[A ~>> B](ab => a => a >>> ab, Lambda.fromFunction(_))

  implicit val anySchema: Schema[_ ~> _] = Schema[Expression]
    .transform(eval => Lambda.unsafe.attempt(_ => eval), _.compile(CompilationContext.initial))

  def apply[B](b: => B)(implicit schema: Schema[B]): Any ~> B =
    Lambda.unsafe.attempt(_ => Literal(schema.toDynamic(b), schema.ast))

  def fromFunction[A, B](f: => A ~>> B): A ~> B = {
    Lambda.unsafe.attempt { ctx =>
      val key   = Binding(ctx.level)
      val body  = f(Lambda.unsafe.attempt[Any, A](_ => Lookup(key))).compile(ctx.next)
      val input = Identity
      FunctionDef(key, body, input)
    }
  }

  def identity[A]: A ~> A = Lambda.unsafe.attempt[A, A](_ => Identity)

  object dynamic {
    def jsonTransform(jsonT: JsonT): DynamicValue ~> DynamicValue =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.JsonTransform(jsonT)))

    def path(p: String*): DynamicValue ~> Option[DynamicValue] =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.Path(p.toList, false)))

    def pathSeq(p: String*): DynamicValue ~> Option[DynamicValue] =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.Path(p.toList, true)))

    def toDynamic[A](implicit schema: Schema[A]): A ~> DynamicValue =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.ToDynamic(schema.ast)))

    def toTyped[A](implicit schema: Schema[A]): DynamicValue ~> Option[A] =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.Typed(schema.ast)))
  }

  object dict {
    def get[A, K, V](key: A ~> K, map: A ~> Map[K, V]): A ~> Option[V] =
      Lambda.unsafe.attempt(ctx => Dict(Dict.Get(key.compile(ctx), map.compile(ctx))))

    def put[A, K, V](key: A ~> K, value: A ~> V, map: A ~> Map[K, V]): A ~> Map[K, V] =
      Lambda.unsafe.attempt(ctx => Dict(Dict.Put(key.compile(ctx), value.compile(ctx), map.compile(ctx))))

    def toPair[K, V]: Map[K, V] ~> List[(K, V)] = Lambda.unsafe.attempt(_ => Dict(Dict.ToPair))
  }

  object option {
    def apply[A, B](ab: Option[A ~> B]): A ~> Option[B] =
      Lambda.unsafe.attempt(ctx => Opt(Opt.Apply(ab.map(_.compile(ctx)))))

    def fold[A, B, C](opt: A ~> Option[B], ifNone: A ~> C, ifSome: B ~> C): A ~> C =
      Lambda.unsafe.attempt(ctx => Opt(Opt.Fold(opt.compile(ctx), ifNone.compile, ifSome.compile(ctx))))

  }

  object unsafe {
    def attempt[A, B](eval: CompilationContext => Expression): A ~> B =
      new Lambda[A, B] {
        override def compile(context: CompilationContext): Expression = eval(context)
      }
    def fromEndpoint(endpoint: Endpoint): DynamicValue ~> DynamicValue =
      Lambda.unsafe.attempt(_ => Unsafe(Unsafe.EndpointCall(endpoint)))
  }
}

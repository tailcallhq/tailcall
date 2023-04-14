package tailcall.runtime.lambda

import tailcall.runtime.JsonT
import tailcall.runtime.lambda.Expression._
import tailcall.runtime.model.Endpoint
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service.EvaluationContext.Binding
import tailcall.runtime.service.EvaluationRuntime
import zio.ZIO
import zio.schema.{DynamicValue, Schema}

sealed trait Lambda[-A, +B] {
  self =>
  final def <<<[C](other: C ~> A): C ~> B = other >>> self

  final def apply[A2, A1 <: A](r: A2 ~> A1): A2 ~> B = r >>> self

  def compile(context: CompilationContext): Expression

  final def compose[C](other: C ~> A): C ~> B = other >>> self

  final def debug(prefix: String): A ~> B = self >>> Lambda.unsafe.debug(prefix)

  override def equals(obj: Any): Boolean = {
    obj match {
      case other: Lambda[_, _] => self.compile == other.compile
      case _                   => false
    }
  }

  final def compile: Expression = compile(CompilationContext.initial)

  final def evaluate[R1 <: A](implicit ev: Any <:< R1): ZIO[EvaluationRuntime with HttpDataLoader, Throwable, B] =
    (self: R1 ~> B).evaluateWith {}

  final def evaluateWith(r: A): ZIO[EvaluationRuntime with HttpDataLoader, Throwable, B] =
    EvaluationRuntime.evaluate(self)(r)

  final def pipe[C](other: B ~> C): A ~> C = self >>> other

  final def >>>[C](other: B ~> C): A ~> C = Lambda.unsafe.attempt(ctx => Pipe(self.compile(ctx), other.compile(ctx)))

  final def toDynamic[B1 >: B](implicit ev: Schema[B1]): A ~> DynamicValue = self >>> Lambda.dynamic.toDynamic[B1]
}

object Lambda {
  def apply[B](b: => B)(implicit schema: Schema[B]): Any ~> B =
    Lambda.unsafe.attempt(_ => Literal(schema.toDynamic(b), schema.ast))

  def fromFunction[A, B](f: => Any ~> A => Any ~> B): A ~> B = {
    Lambda.unsafe.attempt { ctx =>
      val key   = Binding(ctx.level)
      val body  = f(Lambda.unsafe.attempt[Any, A](_ => Lookup(key))).compile(ctx.next)
      val input = Identity
      FunctionDef(key, body, input)
    }
  }

  def identity[A]: A ~> A = Lambda.unsafe.attempt[A, A](_ => Identity)

  def recurse[A, B](f: (A ~> B) => A ~> B): A ~> B =
    Lambda.unsafe.attempt { ctx =>
      val key   = Binding(ctx.level)
      val body  = f(Lambda.unsafe.attempt[A, B](_ => Immediate(Lookup(key)))).compile(ctx.next)
      val input = Defer(body)
      FunctionDef(key, body, input)
    }

  object logic {
    def and[A](left: A ~> Boolean, right: A ~> Boolean): A ~> Boolean =
      Lambda.unsafe.attempt[A, Boolean] { ctx =>
        Logical(Logical.Binary(Logical.Binary.And, left.compile(ctx), right.compile(ctx)))
      }

    def cond[A, B](c: A ~> Boolean)(isTrue: A ~> B, isFalse: A ~> B): A ~> B =
      Lambda.unsafe.attempt[A, B] { ctx =>
        Expression
          .Logical(Logical.Unary(c.compile(ctx), Logical.Unary.Diverge(isTrue.compile(ctx), isFalse.compile(ctx))))
      }

    def eq[A, B](a: A ~> B, b: A ~> B)(implicit ev: Equatable[B]): A ~> Boolean =
      Lambda.unsafe.attempt(ctx => EqualTo(a.compile(ctx), b.compile(ctx), ev.tag))

    def not[A](a: A ~> Boolean): A ~> Boolean =
      Lambda.unsafe.attempt[A, Boolean](ctx => Logical(Logical.Unary(a.compile(ctx), Logical.Unary.Not)))

    def or[A](left: A ~> Boolean, right: A ~> Boolean): A ~> Boolean =
      Lambda.unsafe.attempt[A, Boolean] { ctx =>
        Logical(Logical.Binary(Logical.Binary.Or, left.compile(ctx), right.compile(ctx)))
      }
  }

  object math {
    def dbl[A, B](a: A ~> B)(implicit ev: Numeric[B]): A ~> B = mul(a, inc(ev(ev.one)))

    def inc[A, B](a: A ~> B)(implicit ev: Numeric[B]): A ~> B = add(a, ev(ev.one))

    def add[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Add, a.compile(ctx), b.compile(ctx)), ev.tag))

    def mul[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Multiply, a.compile(ctx), b.compile(ctx)), ev.tag))

    def dec[A, B](a: A ~> B)(implicit ev: Numeric[B]): A ~> B = sub(a, ev(ev.one))

    def sub[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B = add(a, neg(b))

    def neg[A, B](ab: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Unary(Math.Unary.Negate, ab.compile(ctx)), ev.tag))

    def div[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Divide, a.compile(ctx), b.compile(ctx)), ev.tag))

    def gt[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.GreaterThan, a.compile(ctx), b.compile(ctx)), ev.tag))

    def gte[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean =
      Lambda.unsafe
        .attempt(ctx => Math(Math.Binary(Math.Binary.GreaterThanEqual, a.compile(ctx), b.compile(ctx)), ev.tag))

    def mod[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Modulo, a.compile(ctx), b.compile(ctx)), ev.tag))
  }

  object dynamic {
    def jsonTransform(jsonT: JsonT): DynamicValue ~> DynamicValue =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.JsonTransform(jsonT)))

    def path(p: String*): DynamicValue ~> Option[DynamicValue] =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.Path(p.toList)))

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

    def isNone[A]: Option[A] ~> Boolean = Lambda.unsafe.attempt(_ => Opt(Opt.IsNone))

    def isSome[A]: Option[A] ~> Boolean = Lambda.unsafe.attempt(_ => Opt(Opt.IsSome))
  }

  object unsafe {
    def debug[A](prefix: String): A ~> A = Lambda.unsafe.attempt[A, A](_ => Unsafe(Unsafe.Debug(prefix)))

    def die(reason: String): Any ~> Nothing = Lambda.unsafe.attempt(_ => Unsafe(Unsafe.Die(reason)))

    def fromEndpoint(endpoint: Endpoint): DynamicValue ~> DynamicValue =
      Lambda.unsafe.attempt(_ => Unsafe(Unsafe.EndpointCall(endpoint)))

    def attempt[A, B](eval: CompilationContext => Expression): A ~> B =
      new Lambda[A, B] {
        override def compile(context: CompilationContext): Expression = eval(context)
      }
  }

  implicit val anySchema: Schema[_ ~> _] = Schema[Expression]
    .transform(eval => Lambda.unsafe.attempt(_ => eval), _.compile(CompilationContext.initial))

  implicit def schema[A, B]: Schema[A ~> B] = anySchema.asInstanceOf[Schema[A ~> B]]

  implicit def schemaFunction[A, B]: Schema[Any ~> A => Any ~> B] =
    Schema[A ~> B].transform[Any ~> A => Any ~> B](ab => a => a >>> ab, Lambda.fromFunction(_))
}

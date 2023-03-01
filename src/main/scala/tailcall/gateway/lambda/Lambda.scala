package tailcall.gateway.lambda

import tailcall.gateway.lambda.Expression._
import tailcall.gateway.service.EvaluationContext.Binding
import tailcall.gateway.service.EvaluationRuntime
import zio.schema.{DynamicValue, Schema}

sealed trait Lambda[-A, +B] {
  self =>
  final def <<<[C](other: C ~> A): C ~> B = other >>> self

  final def pipe[C](other: B ~> C): A ~> C = self >>> other

  final def >>>[C](other: B ~> C): A ~> C = Lambda.unsafe.attempt(ctx => Pipe(self.compile(ctx), other.compile(ctx)))

  final def compose[C](other: C ~> A): C ~> B = other >>> self

  final def compile: Expression = compile(CompilationContext.initial)

  def compile(context: CompilationContext): Expression

  final def evaluate: LExit[EvaluationRuntime, Throwable, A, B] = EvaluationRuntime.evaluate(self)
}

object Lambda {
  def apply[B](b: => B)(implicit schema: Schema[B]): Any ~> B =
    Lambda.unsafe.attempt(_ => Literal(schema.toDynamic(b), schema.asInstanceOf[Schema[Any]]))

  def fromLambdaFunction[A, B](f: => (Any ~> A) => (Any ~> B)): A ~> B = {
    Lambda.unsafe.attempt { ctx =>
      val key   = Binding(ctx.level)
      val body  = f(Lambda.unsafe.attempt[Any, A](_ => Lookup(key))).compile(ctx.next)
      val input = Identity
      FunctionDef(key, body, input)
    }
  }

  def identity[A]: A ~> A                 = Lambda.unsafe.attempt[A, A](_ => Identity)
  def die(reason: String): Any ~> Nothing = Lambda.unsafe.attempt(_ => Die(reason))
  def debug[A](prefix: String): A ~> A    = Lambda.unsafe.attempt[A, A](_ => Debug(prefix))

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
      Lambda.unsafe.attempt(ctx => EqualTo(a.compile(ctx), b.compile(ctx), ev.any))

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

    def mul[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Multiply, a.compile(ctx), b.compile(ctx)), ev.any))

    def dec[A, B](a: A ~> B)(implicit ev: Numeric[B]): A ~> B = sub(a, ev(ev.one))

    def sub[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B = add(a, neg(b))

    def add[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Add, a.compile(ctx), b.compile(ctx)), ev.any))

    def neg[A, B](ab: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Unary(Math.Unary.Negate, ab.compile(ctx)), ev.any))

    def div[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Divide, a.compile(ctx), b.compile(ctx)), ev.any))

    def gt[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.GreaterThan, a.compile(ctx), b.compile(ctx)), ev.any))

    def gte[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean =
      Lambda.unsafe
        .attempt(ctx => Math(Math.Binary(Math.Binary.GreaterThanEqual, a.compile(ctx), b.compile(ctx)), ev.any))

    def mod[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(Math.Binary.Modulo, a.compile(ctx), b.compile(ctx)), ev.any))
  }

  object dynamic {
    def toTyped[A](implicit schema: Schema[A]): DynamicValue ~> Option[A] =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.Typed(schema.asInstanceOf[Schema[Any]])))

    def path(p: String*): DynamicValue ~> Option[DynamicValue] =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.Path(p.toList)))

    def toDynamic[A](implicit schema: Schema[A]): A ~> DynamicValue =
      Lambda.unsafe.attempt(_ => Dynamic(Dynamic.ToDynamic(schema.asInstanceOf[Schema[Any]])))
  }

  object dict {
    def get[A, K, V](key: A ~> K, map: A ~> Map[K, V]): A ~> Option[V] =
      Lambda.unsafe.attempt(ctx => Dict(Dict.Get(key.compile(ctx), map.compile(ctx))))

    def concat[A, B, K, V](map1: A ~> Map[K, V], map2: B ~> Map[K, V]): A ~> Map[K, V] =
      Lambda.unsafe.attempt(ctx => Dict(Dict.Concatenate(map1.compile(ctx), map2.compile(ctx))))
  }

  object option {
    def isSome[A]: Option[A] ~> Boolean = Lambda.unsafe.attempt(_ => Opt(Opt.IsSome))

    def isNone[A]: Option[A] ~> Boolean = Lambda.unsafe.attempt(_ => Opt(Opt.IsNone))

    def fold[A, B, C](opt: A ~> Option[B], ifNone: A ~> C, ifSome: B ~> C): A ~> C =
      Lambda.unsafe.attempt(ctx => Opt(Opt.Fold(opt.compile(ctx), ifNone.compile, ifSome.compile(ctx))))

    def apply[A, B](ab: Option[A ~> B]): A ~> Option[B] =
      Lambda.unsafe.attempt(ctx => Opt(Opt.Apply(ab.map(_.compile(ctx)))))
  }

  object unsafe {
    def attempt[A, B](eval: CompilationContext => Expression): A ~> B =
      new Lambda[A, B] {
        override def compile(context: CompilationContext): Expression = eval(context)
      }
  }

  implicit val anySchema: Schema[_ ~> _] = Schema[Expression]
    .transform(eval => Lambda.unsafe.attempt(_ => eval), _.compile(CompilationContext.initial))

  implicit def schema[A, B]: Schema[A ~> B] = anySchema.asInstanceOf[Schema[A ~> B]]
}

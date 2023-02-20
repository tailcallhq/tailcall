package tailcall.gateway.lambda

import tailcall.gateway.lambda.DynamicEval.{EqualTo, Literal, Logical, Math}
import tailcall.gateway.lambda.EvaluationContext.Binding
import tailcall.gateway.remote.Remote
import zio.schema.Schema

sealed trait Lambda[-A, +B] {
  self =>
  final def >>>[C](other: B ~> C): A ~> C =
    Lambda.unsafe.attempt(ctx => DynamicEval.Pipe(self.compile(ctx), other.compile(ctx)))

  def compile(context: CompilationContext): DynamicEval

  final def evaluate: LExit[LambdaRuntime, Throwable, A, B] = LambdaRuntime.evaluate(self)
}

object Lambda {
  object logic {
    def and[A](left: A ~> Boolean, right: A ~> Boolean): A ~> Boolean =
      Lambda.unsafe.attempt[A, Boolean] { ctx =>
        DynamicEval.Logical(Logical.Binary(left.compile(ctx), right.compile(ctx), Logical.Binary.And))
      }

    def diverge[A, B](cond: A ~> Boolean, isTrue: A ~> B, isFalse: A ~> B): A ~> B =
      Lambda.unsafe.attempt[A, B] { ctx =>
        DynamicEval
          .Logical(Logical.Unary(cond.compile(ctx), Logical.Unary.Diverge(isTrue.compile(ctx), isFalse.compile(ctx))))
      }

    def not[A](a: A ~> Boolean): A ~> Boolean =
      Lambda.unsafe.attempt[A, Boolean](ctx => DynamicEval.Logical(Logical.Unary(a.compile(ctx), Logical.Unary.Not)))

    def or[A](left: A ~> Boolean, right: A ~> Boolean): A ~> Boolean =
      Lambda.unsafe.attempt[A, Boolean] { ctx =>
        DynamicEval.Logical(Logical.Binary(left.compile(ctx), right.compile(ctx), Logical.Binary.Or))
      }

  }

  def apply[A, B](b: B)(implicit ctor: Constructor[B]): A ~> B =
    Lambda.unsafe.attempt(_ => Literal(ctor.schema.toDynamic(b), ctor.asInstanceOf[Constructor[Any]]))

  object math {
    def subtract[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B = add(a, negate(b))

    def add[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B    =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Add), ev.any))
    def divide[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Divide), ev.any))

    def modulo[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Modulo), ev.any))

    def multiply[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Multiply), ev.any))

    def gt[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.GreaterThan), ev.any))

    def negate[A, B](ab: A ~> B)(implicit ev: Numeric[B]): A ~> B =
      Lambda.unsafe.attempt(ctx => Math(Math.Unary(ab.compile(ctx), Math.Unary.Negate), ev.any))
  }

  def equalTo[A, B](a: A ~> B, b: A ~> B)(implicit ev: Equatable[B]): A ~> Boolean =
    Lambda.unsafe.attempt(ctx => EqualTo(a.compile(ctx), b.compile(ctx), ev.any))

  def identity[A]: A ~> A = Lambda.unsafe.attempt[A, A](_ => DynamicEval.Identity)

  // TODO: add unit test
  def fromFunction[A, B](f: Remote[A] => Remote[B]): A ~> B = {
    Lambda.unsafe.attempt { ctx =>
      val key  = Binding(ctx.next.level)
      val body = f(Remote(Lambda.unsafe.attempt[Any, A](_ => DynamicEval.Lookup(key)))).toLambda
      DynamicEval.FunctionDef(key, body.compile(ctx))
    }
  }

  object unsafe {
    object attempt {
      def apply[A, B](eval: CompilationContext => DynamicEval): A ~> B =
        new Lambda[A, B] {
          override def compile(context: CompilationContext): DynamicEval = eval(context)
        }
    }
  }

  implicit val anySchema: Schema[_ ~> _] = Schema[DynamicEval]
    .transform(eval => Lambda.unsafe.attempt(_ => eval), _.compile(CompilationContext.initial))

  implicit def schema[A, B]: Schema[A ~> B] = anySchema.asInstanceOf[Schema[A ~> B]]
}

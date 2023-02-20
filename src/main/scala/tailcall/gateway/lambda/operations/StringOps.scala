package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.StringOperations
import tailcall.gateway.lambda.Remote

trait StringOps {
  implicit final class RemoteStringOps(val self: Remote[String]) {
    def ++(other: Remote[String]): Remote[String] =
      Remote
        .unsafe
        .attempt(ctx =>
          StringOperations(
            StringOperations.Concat(self.compile(ctx), other.compile(ctx))
          )
        )
  }

  implicit final class ComposeStringInterpolator(val sc: StringContext) {
    def rs[A](args: (Remote[String])*): Remote[String] = {
      val strings             = sc.parts.iterator
      val seq                 = args.iterator
      var buf: Remote[String] = Remote(strings.next())
      while (strings.hasNext) buf = buf ++ seq.next() ++ Remote(strings.next())
      buf
    }
  }
}

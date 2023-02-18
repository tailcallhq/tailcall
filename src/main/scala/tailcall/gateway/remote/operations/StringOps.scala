package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait StringOps {
  implicit final class RemoteStringOps(val self: Remote[String]) {
    def ++(other: Remote[String]): Remote[String] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.concatStrings(self.compile(ctx), other.compile(ctx))
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

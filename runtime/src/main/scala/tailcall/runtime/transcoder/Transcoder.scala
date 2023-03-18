package tailcall.runtime.transcoder

/**
 * A transcoder is a function that takes an A and returns a
 * B, or an error. It can be composed using the >>> operator
 * with other transcoders to create a pipeline. A transcoder
 * between A ~> C can be derived provided there exists a B
 * such that a transcoder from A ~> B exists and a
 * transcoder from B ~> C already exists.
 */
final case class Transcoder[-A, +B](run: A => Transcoder.Output[B]) {
  self =>
  def apply(a: A): Transcoder.Output[B] = run(a)

  def >>>[C](other: Transcoder[B, C]): Transcoder[A, C] = Transcoder(self(_).flatMap(other(_)))
}

object Transcoder extends OrcToBlueprint {
  implicit final class TranscoderOps[A](private val a: A) extends AnyVal {
    def transcodeTo[B](implicit ev: TranscoderLookup[A, B]): Output[B] = ev.transcoder.run(a)
  }

  def collect[A]: PartiallyAppliedCollect[A] = new PartiallyAppliedCollect[A]()

  def collectEither[A]: PartiallyAppliedCollectEither[A] = new PartiallyAppliedCollectEither[A]()

  final class PartiallyApplied[A, C] {
    def apply[B](implicit from: Transcoder[A, B], to: Transcoder[B, C]): Transcoder[A, C] =
      Transcoder(a => from(a).flatMap(to(_)))
  }

  final class PartiallyAppliedCollect[A] {
    def apply[B](pf: PartialFunction[A, B]): Transcoder[A, B] = Transcoder(a => Output.fromOption(pf.lift(a)))
  }

  final class PartiallyAppliedCollectEither[A] {
    def apply[B](pf: PartialFunction[A, Either[String, B]]): Transcoder[A, B] =
      Transcoder(a =>
        pf.lift(a) match {
          case Some(value) => Output.fromEither(value)
          case None        => Output.empty
        }
      )
  }

  sealed trait Output[+A] {
    self =>
    def flatMap[B](ab: A => Output[B]): Output[B] = self.fold(Output.empty, Output.error(_), ab)

    def fold[B](isEmpty: => B, isError: String => B, isSucceed: A => B): B =
      self match {
        case Output.Empty          => isEmpty
        case Output.Error(message) => isError(message)
        case Output.Succeed(value) => isSucceed(value)
      }

    def orElse[A1 >: A](other: Output[A1]): Output[A1] = self.fold[Output[A1]](other, _ => other, Output.succeed(_))
  }
  object Output           {
    def empty: Output[Nothing] = Empty

    def error(message: String): Output[Nothing] = Error(message)

    def succeed[A](value: A): Output[A] = Succeed(value)

    def fromOption[A](option: Option[A]): Output[A] = option.fold[Output[A]](Empty)(Succeed(_))

    def fromEither[A](either: Either[String, A]): Output[A] = either.fold[Output[A]](Error(_), Succeed(_))

    final case class Error(message: String) extends Output[Nothing]

    final case class Succeed[A](value: A) extends Output[A]

    case object Empty extends Output[Nothing]
  }
}

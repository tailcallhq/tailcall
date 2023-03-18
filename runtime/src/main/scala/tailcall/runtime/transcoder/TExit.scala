package tailcall.runtime.transcoder

sealed trait TExit[+A] {
  self =>

  def map[B](ab: A => B): TExit[B] = self.flatMap(a => TExit.succeed(ab(a)))

  def flatMap[B](ab: A => TExit[B]): TExit[B] = self.fold(TExit.empty, TExit.fail(_), ab)

  def orElse[A1 >: A](other: TExit[A1]): TExit[A1] = self.fold[TExit[A1]](other, _ => other, TExit.succeed(_))

  def <>[A1 >: A](other: TExit[A1]): TExit[A1] = self orElse other

  def toEither: Either[String, A] = self.fold[Either[String, A]](Left("Empty"), Left(_), Right(_))

  def toOption: Option[A] = self.fold[Option[A]](None, _ => None, Some(_))

  def fold[B](isEmpty: => B, isError: String => B, isSucceed: A => B): B =
    self match {
      case TExit.Empty            => isEmpty
      case TExit.Failure(message) => isError(message)
      case TExit.Succeed(value)   => isSucceed(value)
    }

  def getOrElse[A1 >: A](default: => A1): A1 = self.fold[A1](default, _ => default, identity)
}

object TExit {
  def empty: TExit[Nothing] = Empty

  def fail(message: String): TExit[Nothing] = Failure(message)

  def succeed[A](value: A): TExit[A] = Succeed(value)

  def fromOption[A](option: Option[A]): TExit[A] = option.fold[TExit[A]](Empty)(Succeed(_))

  def foreach[A, B](list: List[A])(f: A => TExit[B]): TExit[List[B]] =
    list.foldRight[TExit[List[B]]](succeed(Nil))((exit, output) => output.flatMap(list => f(exit) map (_ :: list)))

  def fromEither[A](either: Either[String, A]): TExit[A] = either.fold[TExit[A]](Failure(_), Succeed(_))

  final case class Failure(message: String) extends TExit[Nothing]

  final case class Succeed[A](value: A) extends TExit[A]

  case object Empty extends TExit[Nothing]
}

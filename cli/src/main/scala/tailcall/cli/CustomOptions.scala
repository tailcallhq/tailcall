package tailcall.cli

import tailcall.runtime.ast.Digest
import zio.cli._
import zio.http.URL

object CustomOptions {
  def digest(name: String): Options[Digest] =
    Options.text(name).mapOrFail { digest =>
      if ("^[a-fA-F0-9]{64}$".r.matches(digest)) Right(Digest.fromHex(digest))
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Digest must be a SHA-256 hash.")))
    }

  def integer(name: String): Options[Int] =
    Options.text(name).mapOrFail { int =>
      if ("^[0-9]+$".r.matches(int)) Right(int.toInt)
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Integer must be a positive number.")))
    }

  def url(name: String): Options[URL] =
    Options.text(name).mapOrFail { string =>
      URL.fromString(string) match {
        case Left(_) => Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p(s"Invalid URL: ${string}")))
        case Right(value) => Right(value)
      }
    }
}

package tailcall.cli

import zio.cli._

object CustomOptions {
  def digest(name: String): Options[String] =
    Options.text(name).mapOrFail { digest =>
      if ("^[a-fA-F0-9]{64}$".r.matches(digest)) Right(digest)
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Digest must be a SHA-256 hash.")))
    }
}

package tailcall.cli

import zio.cli._

object CustomOptions {
  def digest(name: String): Options[String] =
    Options.text(name).mapOrFail { digest =>
      if (digest.length == 64) Right(digest)
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Digest must be 64 characters long")))
    }
}

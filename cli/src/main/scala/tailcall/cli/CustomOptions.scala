package tailcall.cli

import tailcall.runtime.ast.Digest
import tailcall.runtime.ast.Digest.Algorithm
import zio.cli._

object CustomOptions {
  def digest(name: String): Options[Digest] =
    Options.text(name).mapOrFail { digest =>
      if ("^[a-fA-F0-9]{64}$".r.matches(digest)) Right(Digest.fromHex(Algorithm.SHA_256, digest))
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Digest must be a SHA-256 hash.")))
    }
}

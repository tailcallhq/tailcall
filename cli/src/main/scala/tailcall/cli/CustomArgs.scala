package tailcall.cli

import tailcall.runtime.ast.Digest
import zio.cli.{Args, HelpDoc}

object CustomArgs {
  def digestArgs: Args[Digest]               = CustomArgs.digestArgs("digest")
  def digestArgs(name: String): Args[Digest] =
    Args.text(name).mapOrFail { digest =>
      if ("^[a-fA-F0-9]{64}$".r.matches(digest)) Right(Digest.fromHex(digest))
      else Left(HelpDoc.p("Digest must be a SHA-256 hash."))
    }
}

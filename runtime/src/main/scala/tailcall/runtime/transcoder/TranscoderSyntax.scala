package tailcall.runtime.transcoder

trait TranscoderSyntax {
  implicit final class Syntax[A](private val a: A) {
    def transcode[B](implicit ev: TranscoderLookup[A, B]): TExit[B] = ev.transcoder.run(a)
  }
}

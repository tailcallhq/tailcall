package tailcall.runtime.transcoder

trait TranscoderLookup[A, B] {
  def transcoder: Transcoder[A, B]
}

object TranscoderLookup {
  implicit def baseCase[A, B](implicit ev: Transcoder[A, B]): TranscoderLookup[A, B] =
    new TranscoderLookup[A, B] {
      override def transcoder: Transcoder[A, B] = ev
    }

  implicit def inductiveCase[A, B, C](implicit from: Transcoder[A, B], to: Transcoder[B, C]): TranscoderLookup[A, C] =
    new TranscoderLookup[A, C] {
      override def transcoder: Transcoder[A, C] = from >>> to
    }
}

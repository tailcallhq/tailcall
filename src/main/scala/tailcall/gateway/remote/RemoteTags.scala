package tailcall.gateway.remote

trait RemoteTags {
  sealed trait EqualityTag[A] {
    def equals(left: A, right: A): Boolean
  }

  object EqualityTag {
    implicit val intTag: EqualityTag[Int] = new EqualityTag[Int] {
      override def equals(left: Int, right: Int): Boolean = left == right
    }

    implicit val stringTag: EqualityTag[String] = new EqualityTag[String] {
      override def equals(left: String, right: String): Boolean = left == right
    }

    implicit val booleanTag: EqualityTag[Boolean] = new EqualityTag[Boolean] {
      override def equals(left: Boolean, right: Boolean): Boolean = left == right
    }
  }

  sealed trait NumericTag[A] {
    def add(left: A, right: A): A
    def negate(value: A): A
    def multiply(left: A, right: A): A
    def divide(left: A, right: A): A
    def modulo(left: A, right: A): A
    def one: A
  }

  object NumericTag {
    implicit val intTag: NumericTag[Int] = new NumericTag[Int] {
      override def add(left: Int, right: Int): Int      = left + right
      override def negate(value: Int): Int              = -value
      override def multiply(left: Int, right: Int): Int = left * right
      override def divide(left: Int, right: Int): Int   = left / right
      override def modulo(left: Int, right: Int): Int   = left % right
      override def one: Int                             = 1
    }
  }

  sealed trait IndexSeqTag[A]
  object IndexSeqTag {
    implicit val intTag: IndexSeqTag[Int] = new IndexSeqTag[Int] {}
  }
}

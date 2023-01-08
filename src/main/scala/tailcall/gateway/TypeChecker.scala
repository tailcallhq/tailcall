package tailcall.gateway

import tailcall.gateway.adt.Config
import caliban.parsing.adt.Document

final case class TypeChecker() {
  def check(config: Config, document: Document): Either[Error, Unit] = ???
}

object TypeChecker {
   
}

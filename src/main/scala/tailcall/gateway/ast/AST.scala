package tailcall.gateway.ast

/**
 * Contains all the resolver AST
 */

object AST {
  sealed trait GResolver
  object GResolver {
    final case class GProperty(name: String)                                  extends GResolver
    case object GIdentity                                                     extends GResolver
    final case class GObject(name: String, fields: List[(String, GResolver)]) extends GResolver
    final case class GHttp(url: String)                                       extends GResolver
  }
}

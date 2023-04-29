package tailcall.runtime.model

import tailcall.runtime.http.Method

trait HttpOperation {
  def body: Option[String]

  def compress: HttpOperation

  final def compressMethod: Option[Method] =
    method match {
      case Some(Method.GET) => None
      case method           => method
    }

  final def compressQuery: Option[Map[String, List[String]]] =
    query match {
      case Some(query) if query.isEmpty => None
      case query                        => query
    }

  def input: Option[TSchema]

  def method: Option[Method]

  def output: Option[TSchema]

  def path: Path

  def query: Option[Map[String, List[String]]]
}

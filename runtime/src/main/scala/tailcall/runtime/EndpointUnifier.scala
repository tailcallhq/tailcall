package tailcall.runtime

import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Endpoint
import zio.Chunk

object EndpointUnifier {
  def unify(endpoints: List[Endpoint]): TValid[String, List[Endpoint]] =
    TValid.foreach(endpoints.groupBy(endpoint => (endpoint.address, endpoint.method, endpoint.path)).toList) {
      case ((addr, method, path), l) =>
        val headers = Chunk.fromIterable(l.flatMap(_.headers.toList))
        for {
          input  <- SchemaUnifier.unify(l.flatMap(_.input.toList))
          output <- SchemaUnifier.unify(l.flatMap(_.output.toList))
        } yield Endpoint(address = addr).withMethod(method).withPath(path).withInput(input).withOutput(output)
          .withHeader(headers: _*)
    }
}

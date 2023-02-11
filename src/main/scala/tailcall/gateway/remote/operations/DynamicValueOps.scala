package tailcall.gateway.remote.operations

import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def path(fields: String*): Remote[Option[DynamicValue]] = ???
    def asString: Remote[Option[String]]                    = ???
    def asBoolean: Remote[Option[Boolean]]                  = ???
    def asInt: Remote[Option[Int]]                          = ???
    def asLong: Remote[Option[Long]]                        = ???
    def asDouble: Remote[Option[Double]]                    = ???
    def asFloat: Remote[Option[Float]]                      = ???
    def asList: Remote[Option[List[DynamicValue]]]          = ???
    def asMap: Remote[Option[Map[String, DynamicValue]]]    = ???
  }
}

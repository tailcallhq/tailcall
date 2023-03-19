package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc
import zio.json.ast.Json
import zio.schema.DynamicValue

package object transcoder {
  implicit def orc2Blueprint: Transcoder[Orc, String, Blueprint]                           = ???
  implicit def config2Blueprint: Transcoder[Config, Nothing, Blueprint]                    = ???
  implicit def dynamicValue2JsonAST: Transcoder[DynamicValue, String, Json]                = ???
  implicit def json2DynamicValue: Transcoder[Json, String, DynamicValue]                   = ???
  implicit def responseValue2DynamicValue: Transcoder[ResponseValue, String, DynamicValue] = ???
  implicit def inputValue2DynamicValue: Transcoder[InputValue, String, DynamicValue]       = ???
  implicit def primitive2Value[A]: Transcoder[DynamicValue.Primitive[A], Nothing, Value]   = ???
  implicit def dynamicValue2InputValue: Transcoder[DynamicValue, String, InputValue]       = ???
  implicit def dynamicValue2ResponseValue: Transcoder[DynamicValue, String, ResponseValue] = ???
}

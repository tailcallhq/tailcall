package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc
import zio.json.ast.Json
import zio.schema.DynamicValue

package object transcoder {
  implicit final def orc2Blueprint: Transcoder[Orc, String, Blueprint]                           = ???
  implicit final def config2Blueprint: Transcoder[Config, Nothing, Blueprint]                    = ???
  implicit final def dynamicValue2JsonAST: Transcoder[DynamicValue, String, Json]                = ???
  implicit final def json2DynamicValue: Transcoder[Json, String, DynamicValue]                   = ???
  implicit final def responseValue2DynamicValue: Transcoder[ResponseValue, String, DynamicValue] = ???
  implicit final def inputValue2DynamicValue: Transcoder[InputValue, String, DynamicValue]       = ???
  implicit final def primitive2Value[A]: Transcoder[DynamicValue.Primitive[A], Nothing, Value]   = ???
  implicit final def dynamicValue2InputValue: Transcoder[DynamicValue, String, InputValue]       = ???
  implicit final def dynamicValue2ResponseValue: Transcoder[DynamicValue, String, ResponseValue] = ???
}

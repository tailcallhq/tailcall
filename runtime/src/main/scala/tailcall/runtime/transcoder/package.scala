package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc
import zio.json.ast.Json
import zio.schema.DynamicValue

package object transcoder {
  implicit val orc2Blueprint: Transcoder[Orc, String, Blueprint]                           = Orc2Blueprint
  implicit val config2Blueprint: Transcoder[Config, Nothing, Blueprint]                    = Config2Blueprint
  implicit val dynamicValue2JsonAST: Transcoder[DynamicValue, String, Json]                = DynamicValue2JsonAST
  implicit val json2DynamicValue: Transcoder[Json, String, DynamicValue]                   = Json2DynamicValue
  implicit val responseValue2DynamicValue: Transcoder[ResponseValue, String, DynamicValue] = ResponseValue2DynamicValue
  implicit val inputValue2DynamicValue: Transcoder[InputValue, String, DynamicValue]       = InputValue2DynamicValue
  implicit def primitive2Value[A]: Transcoder[DynamicValue.Primitive[A], Nothing, Value]   = Primitive2Value
  implicit val dynamicValue2InputValue: Transcoder[DynamicValue, String, InputValue]       = DynamicValue2InputValue
  implicit val dynamicValue2ResponseValue: Transcoder[DynamicValue, String, ResponseValue] = DynamicValue2ResponseValue
}

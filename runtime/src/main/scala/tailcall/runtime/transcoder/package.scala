package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc
import zio.json.ast.Json
import zio.schema.DynamicValue

package object transcoder {
  implicit val orc2Blueprint: Transcoder[Orc, String, Blueprint] = Transcoder.fromExit(Orc2Blueprint.toBlueprint)

  implicit val config2Blueprint: Transcoder[Config, Nothing, Blueprint] = Transcoder.total(Config2Blueprint.toBlueprint)

  implicit val dynamicValue2JsonAST: Transcoder[DynamicValue, String, Json] = Transcoder
    .fromExit(DynamicValue2JsonAST.toJson)

  implicit val json2DynamicValue: Transcoder[Json, Nothing, DynamicValue] = Transcoder.total(Json2DynamicValue.fromJson)

  implicit val responseValue2DynamicValue: Transcoder[ResponseValue, Nothing, DynamicValue] = Transcoder
    .total(ResponseValue2DynamicValue.fromResponseValue)

  implicit val inputValue2DynamicValue: Transcoder[InputValue, Nothing, DynamicValue] = Transcoder
    .total(InputValue2DynamicValue.fromInputValue)

  implicit def primitive2Value[A]: Transcoder[DynamicValue.Primitive[A], Nothing, Value] =
    Transcoder.total(Primitive2Value.toValue)

  implicit val dynamicValue2InputValue: Transcoder[DynamicValue, Nothing, InputValue] = Transcoder
    .total(DynamicValue2InputValue.toInputValue)

  implicit val dynamicValue2ResponseValue: Transcoder[DynamicValue, Nothing, ResponseValue] = Transcoder
    .total(DynamicValue2ResponseValue.toResponseValue)
}

package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc
import zio.json.ast.Json
import zio.schema.DynamicValue

package object transcoder extends TranscoderSyntax {
  implicit val orc2Blueprint: Transcoder[Orc, Blueprint] = Transcoder
    .fromExit[Orc, Blueprint](Orc2Blueprint.toBlueprint)

  implicit val config2Blueprint: Transcoder[Config, Blueprint] = Transcoder.total(Config2Blueprint.toBlueprint)

  implicit val dynamicValue2JsonAST: Transcoder[DynamicValue, Json] = Transcoder.fromExit(DynamicValue2JsonAST.toJson)

  implicit val json2DynamicValue: Transcoder[Json, DynamicValue] = Transcoder.total(Json2DynamicValue.fromJson)

  implicit val responseValue2DynamicValue: Transcoder[ResponseValue, DynamicValue] = Transcoder
    .total(ResponseValue2DynamicValue.fromResponseValue)

  implicit val inputValue2DynamicValue: Transcoder[caliban.InputValue, DynamicValue] = Transcoder
    .total(InputValue2DynamicValue.fromInputValue)

  implicit def primitive2Value[A]: Transcoder[DynamicValue.Primitive[A], Value] =
    Transcoder.total(Primitive2Value.toValue)

  implicit val dynamicValue2InputValue: Transcoder[DynamicValue, InputValue] = Transcoder
    .total(DynamicValue2InputValue.toInputValue)

  implicit val dynamicValue2ResponseValue: Transcoder[DynamicValue, ResponseValue] = Transcoder
    .total(DynamicValue2ResponseValue.toResponseValue)
}

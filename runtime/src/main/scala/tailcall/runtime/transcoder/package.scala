package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc
import zio.json.ast.Json
import zio.schema.DynamicValue

package object transcoder {
  implicit val orc2Blueprint: Transcoder[Orc, String, Blueprint] = Transcoder.fromExit(Orc2Blueprint.toBlueprint)

  implicit val config2Blueprint: Transcoder[Config, Nothing, Blueprint] = Transcoder
    .total(Config2Blueprint(_).toBlueprint)

  implicit val dynamicValue2JsonAST: Transcoder[DynamicValue, String, Json] = Transcoder
    .fromExit(DynamicValue2JsonAST.toJson)

  implicit val json2DynamicValue: Transcoder[Json, String, DynamicValue] = Transcoder
    .fromExit(Json2DynamicValue.fromJson)

  implicit val responseValue2DynamicValue: Transcoder[ResponseValue, String, DynamicValue] = Transcoder
    .fromExit(ResponseValue2DynamicValue.fromResponseValue)

  implicit val inputValue2DynamicValue: Transcoder[InputValue, String, DynamicValue] = Transcoder
    .fromExit(InputValue2DynamicValue.fromInputValue)

  implicit def primitive2Value[A]: Transcoder[DynamicValue.Primitive[A], Nothing, Value] =
    Transcoder.total(Primitive2Value.toValue)

  implicit val dynamicValue2InputValue: Transcoder[DynamicValue, String, InputValue] = Transcoder
    .fromExit(DynamicValue2InputValue.toInputValue)

  implicit val dynamicValue2ResponseValue: Transcoder[DynamicValue, String, ResponseValue] = Transcoder
    .fromExit(DynamicValue2ResponseValue.toResponseValue)
}

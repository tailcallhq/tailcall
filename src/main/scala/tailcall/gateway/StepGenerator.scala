package tailcall.gateway

import caliban.schema.Step
import caliban.{InputValue, ResponseValue}
import tailcall.gateway.ast.{Endpoint, Orc}
import tailcall.gateway.internal.HttpClient
import tailcall.gateway.remote.{Remote, UnsafeEvaluator}
import zio._
import zio.json._
import zio.query.ZQuery
import zio.schema.{DynamicValue, Schema}

final case class StepGenerator(client: HttpClient) {
  implicit val codec: JsonCodec[DynamicValue] = zio.schema.codec.JsonCodec
    .jsonCodec(Schema[DynamicValue])

  def convertInputValue(inputValue: InputValue): DynamicValue = {
    val json = inputValue.toJson
    json.fromJson[DynamicValue].getOrElse(???)
  }

  def convertInput(map: Map[String, InputValue]): Map[String, DynamicValue] = map
    .map { case (key, value) => key -> convertInputValue(value) }

  def convertToResponse(dynamicValue: DynamicValue): ResponseValue = {
    val json = dynamicValue.toJson
    json.fromJson[ResponseValue].getOrElse(???)
  }

  def resolveEndpoint(req: Endpoint): Task[DynamicValue] = ???

  def generate(orc: Orc): Step[Any] = orc match {
    case Orc.EndpointOrc(endpoint) => Step.QueryStep(
        ZQuery.fromZIO(resolveEndpoint(endpoint).map(dv => Step.PureStep(convertToResponse(dv))))
      )

    case Orc.FunctionOrc(orc) => Step
        .FunctionStep(input => generate(Orc.remote(orc.toFunction(Remote(convertInput(input))))))

    case Orc.ListOrc(orcs) => Step.ListStep(orcs.map(generate(_)))

    case Orc.ObjectOrc(name, fields) => Step
        .ObjectStep(name, fields.map { case (k, v) => (k, generate(v)) })

    case Orc.RemoteOrc(remote) =>
      val result = UnsafeEvaluator.make().evaluate(remote.compile).asInstanceOf[DynamicValue]

      Step.PureStep(convertToResponse(result))

  }
}

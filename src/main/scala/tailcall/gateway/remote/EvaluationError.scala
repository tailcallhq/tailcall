package tailcall.gateway.remote

import zio.schema.{DynamicValue, Schema}

sealed trait EvaluationError extends Throwable {
  self =>
  override def getMessage(): String = EvaluationError.getMessage(self)
}

object EvaluationError {
  final case class FieldNotFound(name: String) extends EvaluationError

  final case class UnsupportedOperation(operation: String, value: DynamicValue)
      extends EvaluationError

  final case class TypeError(value: DynamicValue, cause: String, schema: Schema[_])
      extends EvaluationError

  final case class BindingNotFound(id: Int) extends EvaluationError

  final case class Death(message: String) extends EvaluationError

  final case class DecodingError(str: String) extends EvaluationError

  final case class InvalidTupleSize(length: Int) extends EvaluationError

  def getMessage(self: EvaluationError): String =
    self match {
      case FieldNotFound(name)                    => s"Field not found: $name"
      case UnsupportedOperation(operation, value) => s"Unsupported operation: $operation on $value"
      case TypeError(value, cause, schema) => s"Type conversion error: $value, $cause, $schema"
      case BindingNotFound(id)             => s"Binding not found: $id"
      case Death(message)                  => s"Died because of: $message"
      case DecodingError(str)              => s"Decoding error: $str"
      case InvalidTupleSize(length)        => s"Invalid tuple size: $length"
    }

}

package tailcall.runtime.transcoder

import caliban.{InputValue, Value}
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.transcoder.Transcoder.Syntax
import zio.schema.DynamicValue

object DynamicValue2InputValue {

  def toInputValue(input: DynamicValue): InputValue = {
    input match {
      case DynamicValue.Sequence(values)        => InputValue.ListValue(values.map(toInputValue).toList)
      case input @ DynamicValue.Primitive(_, _) => input.transcode[Value]
      case DynamicValue.Dictionary(chunks)      => InputValue.ObjectValue(chunks.map { case (k, v) =>
          // TODO: use TExit.fail
          DynamicValueUtil.toTyped[String](k).getOrElse(throw new Error("could not transform")) -> toInputValue(v)
        }.toMap)
      case DynamicValue.Singleton(_)            => Value.NullValue
      case DynamicValue.NoneValue               => Value.NullValue
      case DynamicValue.DynamicAst(_)           => Value.NullValue
      case DynamicValue.SetValue(_)             => Value.NullValue
      case DynamicValue.Record(_, b)            => InputValue.ObjectValue(b.map { case (k, v) => k -> toInputValue(v) })
      case DynamicValue.Enumeration(_, _)       => Value.NullValue
      case DynamicValue.RightValue(_)           => Value.NullValue
      case DynamicValue.SomeValue(input)        => toInputValue(input)
      case DynamicValue.Tuple(_, _)             => Value.NullValue
      case DynamicValue.LeftValue(_)            => Value.NullValue
      case DynamicValue.Error(_)                => Value.NullValue
    }
  }
}

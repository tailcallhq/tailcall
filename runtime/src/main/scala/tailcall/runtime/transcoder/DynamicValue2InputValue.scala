package tailcall.runtime.transcoder

import caliban.{InputValue, Value}
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.transcoder.Transcoder.Syntax
import zio.schema.DynamicValue

object DynamicValue2InputValue extends Transcoder[DynamicValue, String, InputValue] {

  override def run(input: DynamicValue): TExit[String, InputValue] = {
    input match {
      case DynamicValue.Sequence(values)        => TExit.foreach(values.toList)(run(_)).map(InputValue.ListValue(_))
      case input @ DynamicValue.Primitive(_, _) => TExit.succeed(input.transcode[Value])
      case DynamicValue.Dictionary(chunks)      => TExit.foreachChunk(chunks) { case (k, v) =>
          DynamicValueUtil.toTyped[String](k) match {
            case Some(key) => run(v).map(key -> _)
            case None      => TExit.fail("Can not transform Dictionary key to String")
          }
        }.map(entries => InputValue.ObjectValue(entries.toMap))
      case DynamicValue.Singleton(_)            => TExit.fail("Can not transcode Singleton to InputValue")
      case DynamicValue.NoneValue               => TExit.fail("Can not transcode NoneValue to InputValue")
      case DynamicValue.DynamicAst(_)           => TExit.fail("Can not transcode DynamicAst to InputValue")
      case DynamicValue.SetValue(_)             => TExit.fail("Can not transcode SetValue to InputValue")
      case DynamicValue.Record(_, b)            => TExit.foreachIterable(b) { case (k, v) => run(v).map(k -> _) }
          .map(entries => InputValue.ObjectValue(entries.toMap))
      case DynamicValue.Enumeration(_, _)       => TExit.fail("Can not transcode Enumeration to InputValue")
      case DynamicValue.RightValue(_)           => TExit.fail("Can not transcode RightValue to InputValue")
      case DynamicValue.SomeValue(input)        => run(input)
      case DynamicValue.Tuple(_, _)             => TExit.fail("Can not transcode Tuple to InputValue")
      case DynamicValue.LeftValue(_)            => TExit.fail("Can not transcode LeftValue to InputValue")
      case DynamicValue.Error(_)                => TExit.fail("Can not transcode Error to InputValue")
    }
  }
}

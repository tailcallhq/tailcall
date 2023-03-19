package tailcall.runtime.transcoder.data

import caliban.InputValue
import tailcall.runtime.internal.{DynamicValueUtil, TValid}
import tailcall.runtime.transcoder.Transcoder
import zio.schema.DynamicValue

trait ToInputValue {

  final def toInputValue(input: DynamicValue): TValid[String, InputValue] = {
    input match {
      case DynamicValue.Sequence(values) => TValid.foreach(values.toList)(toInputValue(_)).map(InputValue.ListValue(_))
      case DynamicValue.Primitive(input, standardType) => Transcoder.toValue(input, standardType)
      case DynamicValue.Dictionary(chunks)             => TValid.foreachChunk(chunks) { case (k, v) =>
          DynamicValueUtil.toTyped[String](k) match {
            case Some(key) => toInputValue(v).map(key -> _)
            case None      => TValid.fail("Can not transform Dictionary key to String")
          }
        }.map(entries => InputValue.ObjectValue(entries.toMap))
      case DynamicValue.Singleton(_)                   => TValid.fail("Can not transcode Singleton to InputValue")
      case DynamicValue.NoneValue                      => TValid.fail("Can not transcode NoneValue to InputValue")
      case DynamicValue.DynamicAst(_)                  => TValid.fail("Can not transcode DynamicAst to InputValue")
      case DynamicValue.SetValue(_)                    => TValid.fail("Can not transcode SetValue to InputValue")
      case DynamicValue.Record(_, b)      => TValid.foreachIterable(b) { case (k, v) => toInputValue(v).map(k -> _) }
          .map(entries => InputValue.ObjectValue(entries.toMap))
      case DynamicValue.Enumeration(_, _) => TValid.fail("Can not transcode Enumeration to InputValue")
      case DynamicValue.RightValue(_)     => TValid.fail("Can not transcode RightValue to InputValue")
      case DynamicValue.SomeValue(input)  => toInputValue(input)
      case DynamicValue.Tuple(_, _)       => TValid.fail("Can not transcode Tuple to InputValue")
      case DynamicValue.LeftValue(_)      => TValid.fail("Can not transcode LeftValue to InputValue")
      case DynamicValue.Error(_)          => TValid.fail("Can not transcode Error to InputValue")
    }
  }

}

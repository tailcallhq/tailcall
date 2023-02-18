package tailcall.gateway.remote

import tailcall.gateway.remote.DynamicEval._
import zio.schema.Schema

class RemoteSchemaInferer {
  def inferSchema(r: DynamicEval): Schema[_] =
    r match {
      case DynamicEval.Literal(_, ctor)             => ctor.schema
      case DynamicEval.EqualTo(_, _, _)             => Schema[Boolean]
      case DynamicEval.Math(operation, tag)         => operation match {
          case DynamicEval.Math.Binary(_, _, operation) => operation match {
              case DynamicEval.Math.Binary.Add         => tag.schema
              case DynamicEval.Math.Binary.Multiply    => tag.schema
              case DynamicEval.Math.Binary.Divide      => tag.schema
              case DynamicEval.Math.Binary.Modulo      => tag.schema
              case DynamicEval.Math.Binary.GreaterThan => Schema[Boolean]
            }
          case DynamicEval.Math.Unary(_, operation)     =>
            operation match { case DynamicEval.Math.Unary.Negate => tag.schema }
        }
      case DynamicEval.Logical(_)                   => Schema[Boolean]
      case DynamicEval.StringOperations(_)          => Schema[String]
      case DynamicEval.TupleOperations(operation)   => operation match {
          case TupleOperations.GetIndex(value, _) =>
            inferSchema(value) match { case Schema.Tuple2(a, _, _) => a }
          case TupleOperations.Cons(value)        =>
            val a = value.map(inferSchema)
            a.length match {
              case 2 => Schema.tuple2(a(0), a(1))
              case _ => ???
            }
        }
      case DynamicEval.SeqOperations(operation)     => operation match {
          case SeqOperations.Concat(left, _)       => inferSchema(left)
          case SeqOperations.Reverse(seq)          => inferSchema(seq)
          case SeqOperations.Filter(seq, _)        => inferSchema(seq)
          case SeqOperations.FlatMap(_, operation) => inferSchema(operation)
          case SeqOperations.Length(_)             => Schema[Int]
          case SeqOperations.IndexOf(_, _)         => Schema[Int]
          case SeqOperations.Slice(seq, _, _)      => inferSchema(seq)
          case SeqOperations.Head(seq)             => inferSchema(seq)
          case SeqOperations.Sequence(_, ctor)     => Schema.chunk(ctor.schema)
          case SeqOperations.GroupBy(seq, keyFunction) =>
            Schema.map(inferSchema(keyFunction), inferSchema(seq))
        }
      case DynamicEval.MapOperations(_)             => ???
      case DynamicEval.FunctionCall(_, b)           => inferSchema(b)
      case DynamicEval.EitherOperations(operation)  => operation match {
          case EitherOperations.Cons(value)   => value match {
              case Left(v)  => Schema.either(inferSchema(v), Schema[Unit])
              case Right(v) => Schema.either(Schema[Unit], inferSchema(v))
            }
          case EitherOperations.Fold(_, l, _) => inferSchema(l)
        }
      case DynamicEval.ContextOperations(_, _)      => ???
      case DynamicEval.OptionOperations(_)          => ???
      case DynamicEval.EndpointCall(_, _)           => ???
      case DynamicEval.Record(_)                    => ???
      case DynamicEval.Die(_)                       => ???
      case DynamicEval.DynamicValueOperations(_, _) => ???
      case DynamicEval.Debug(_, _)                  => ???
      case DynamicEval.FunctionDef(_, b)            => inferSchema(b)
      case DynamicEval.Lookup(_)                    => ???
      case DynamicEval.Flatten(b)                   => inferSchema(b)
      case DynamicEval.Recurse(b)                   => inferSchema(b)
    }

}
object RemoteSchemaInferer {
  def inferSchema[A](r: Remote[A]): Schema[A] =
    new RemoteSchemaInferer()
      .inferSchema(r.compile(CompilationContext.initial))
      .asInstanceOf[Schema[A]]

  def inferSchema(r: DynamicEval): Schema[_] =
    new RemoteSchemaInferer().inferSchema(r)
}

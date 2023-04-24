package tailcall.runtime

import caliban.InputValue.{ListValue, ObjectValue}
import caliban.Value
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec._
import tailcall.runtime.http.Method
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model.{Path, UnsafeSteps}
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertZIO}

object OperationSpec extends ZIOSpecDefault {
  final case class User(name: String, age: Option[Int])
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("directive")(
      suite("http")(
        test("encoder") {
          val http = Operation
            .Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))

          val actual   = http.toDirective
          val expected = Directive(
            "http",
            Map(
              "path"   -> Value.StringValue(value = "/{{users}}"),
              "method" -> Value.StringValue(value = "POST"),
              "body"   -> Value.StringValue(value = "{{user.id}}"),
            ),
          )

          assertZIO(actual.toZIO)(equalTo(expected))
        },
        test("decoder") {
          val directive = Directive(
            "http",
            Map(
              "path"   -> Value.StringValue(value = "/{{users}}"),
              "method" -> Value.StringValue(value = "POST"),
              "body"   -> Value.StringValue(value = "{{user.id}}"),
            ),
          )

          val actual   = directive.fromDirective[Http]
          val expected = Operation
            .Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))

          assertZIO(actual.toZIO)(equalTo(expected))
        },
      ),
      suite("Steps")(
        test("encoder") {
          val steps: UnsafeSteps = UnsafeSteps(
            Operation
              .Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))
          )
          val actual             = steps.toDirective
          val expected           = Directive(
            "unsafe",
            Map(
              "steps" -> ListValue(List(ObjectValue(Map(
                "http" -> ObjectValue(Map(
                  "path"   -> Value.StringValue(value = "/{{users}}"),
                  "method" -> Value.StringValue(value = "POST"),
                  "body"   -> Value.StringValue(value = "{{user.id}}"),
                ))
              ))))
            ),
          )

          assertZIO(actual.toZIO)(equalTo(expected))
        },
        test("decoder") {
          val directive = Directive(
            "unsafe",
            Map(
              "steps" -> ListValue(List(ObjectValue(Map(
                "http" -> ObjectValue(Map(
                  "path"   -> Value.StringValue(value = "/{{users}}"),
                  "method" -> Value.StringValue(value = "POST"),
                  "body"   -> Value.StringValue(value = "{{user.id}}"),
                ))
              ))))
            ),
          )

          val actual                = directive.fromDirective[UnsafeSteps]
          val expected: UnsafeSteps = UnsafeSteps(
            Operation
              .Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))
          )

          assertZIO(actual.toZIO)(equalTo(expected))
        },
      ),
    )
}

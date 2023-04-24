package tailcall.runtime

import caliban.InputValue.{ListValue, ObjectValue}
import caliban.Value
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec._
import tailcall.runtime.http.Method
import tailcall.runtime.model.Step.Http
import tailcall.runtime.model.{Path, Step, Steps}
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertZIO}

object StepSpec extends ZIOSpecDefault {
  final case class User(name: String, age: Option[Int])
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("directive")(
      suite("http")(
        test("encoder") {
          val http = Step
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
          val expected = Step
            .Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))

          assertZIO(actual.toZIO)(equalTo(expected))
        },
      ),

// FIXME: drop these tests
//      suite("steps")(
//        test("encoder") {
//          val steps: List[Step] = Step.Http(
//            path = Path.empty.withParam("users"),
//            method = Option(Method.POST),
//            body = Option("{{user.id}}"),
//          ) :: Nil
//          val actual            = steps.toDirective
//          val expected          = Directive(
//            "steps",
//            Map(
//              "value" -> ListValue(List(ObjectValue(Map(
//                "http" -> ObjectValue(Map(
//                  "path"   -> Value.StringValue(value = "/{{users}}"),
//                  "method" -> Value.StringValue(value = "POST"),
//                  "body"   -> Value.StringValue(value = "{{user.id}}"),
//                ))
//              ))))
//            ),
//          )
//
//          assertZIO(actual.toZIO)(equalTo(expected))
//        },
//        test("decoder") {
//          val directive = Directive(
//            "steps",
//            Map(
//              "value" -> ListValue(List(ObjectValue(Map(
//                "http" -> ObjectValue(Map(
//                  "path"   -> Value.StringValue(value = "/{{users}}"),
//                  "method" -> Value.StringValue(value = "POST"),
//                  "body"   -> Value.StringValue(value = "{{user.id}}"),
//                ))
//              ))))
//            ),
//          )
//
//          val actual               = directive.fromDirective[List[Step]]
//          val expected: List[Step] = Step.Http(
//            path = Path.empty.withParam("users"),
//            method = Option(Method.POST),
//            body = Option("{{user.id}}"),
//          ) :: Nil
//
//          assertZIO(actual.toZIO)(equalTo(expected))
//        },
//      ) @@ ignore,
      suite("Steps")(
        test("encoder") {
          val steps: Steps = Steps(
            Step.Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))
          )
          val actual       = steps.toDirective
          val expected     = Directive(
            "steps",
            Map(
              "value" -> ListValue(List(ObjectValue(Map(
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
            "steps",
            Map(
              "value" -> ListValue(List(ObjectValue(Map(
                "http" -> ObjectValue(Map(
                  "path"   -> Value.StringValue(value = "/{{users}}"),
                  "method" -> Value.StringValue(value = "POST"),
                  "body"   -> Value.StringValue(value = "{{user.id}}"),
                ))
              ))))
            ),
          )

          val actual          = directive.fromDirective[Steps]
          val expected: Steps = Steps(
            Step.Http(path = Path.empty.withParam("users"), method = Option(Method.POST), body = Option("{{user.id}}"))
          )

          assertZIO(actual.toZIO)(equalTo(expected))
        },
      ),
    )
}

package tailcall.server.internal

import caliban.{GraphQLRequest, InputValue, Value}
import tailcall.test.TailcallSpec
import zio.http.QueryParams
import zio.test.Assertion._
import zio.test._

object GraphQLUtilsSpec extends TailcallSpec {
  override def spec =
    suite("GraphQLUtils")(test("decodeQuery QueryParams") {
      val queryParams = QueryParams.decode(
        """query=query { hello }&extensions={"persistedQuery":{"version":1,"sha256Hash":"7396e63fd4ecca35d454df01c5296efc920ad2d8cc45c5ec876bc3239b8679e3"}}"""
      )
      val expected    = GraphQLRequest(
        query = Some("query { hello }"),
        operationName = None,
        variables = None,
        extensions = Some(Map(
          "persistedQuery" -> InputValue.ObjectValue(fields =
            Map(
              "version"    -> Value.IntValue(1),
              "sha256Hash" -> Value
                .StringValue(value = "7396e63fd4ecca35d454df01c5296efc920ad2d8cc45c5ec876bc3239b8679e3"),
            )
          )
        )),
      )
      val actual      = GraphQLUtils.decodeRequest(queryParams)
      assertZIO(actual)(equalTo(expected))
    })
}

package tailcall.gateway.ast

/**
 * The core AST to represent API orchestration. It takes in
 * an input of type A and performs a series of steps to
 * produce an output of type B.
 */
sealed trait Orch {}

object Orch {
  case class FromEndpoint(endpoint: Endpoint) extends Orch

  def endpoint(endpoint: Endpoint): Orch = FromEndpoint(endpoint)

  // ADD_PATH "/home-page" {
  //   REQ /users {
  //      ADD_PATH "posts" {
  //         REQ /posts/${parent.id}
  //      }
  //   }
  // }
  // 
  
  
}

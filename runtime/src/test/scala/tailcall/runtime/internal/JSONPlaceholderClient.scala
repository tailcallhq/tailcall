package tailcall.runtime.internal

import tailcall.runtime.http.{HttpClient, Method, Request}
import zio.http.Response
import zio.http.model.HttpError
import zio.{ULayer, ZIO, ZLayer}

class JSONPlaceholderClient extends HttpClient {
  val comments = ZIO.readFile(getClass.getResource("comments.json").toURI.getPath).map(Response.json(_))
  val posts    = ZIO.readFile(getClass.getResource("posts.json").toURI.getPath).map(Response.json(_))
  val todos    = ZIO.readFile(getClass.getResource("todos.json").toURI.getPath).map(Response.json(_))
  val users    = ZIO.readFile(getClass.getResource("users.json").toURI.getPath).map(Response.json(_))
  val albums   = ZIO.readFile(getClass.getResource("albums.json").toURI.getPath).map(Response.json(_))
  def userById(id: Int): ZIO[Any, Throwable, Response] =
    id match {
      case 1 => ZIO.readFile(getClass.getResource("userById.json").toURI.getPath).map(Response.json(_))
      case _ => ZIO.succeed(
          Response.fromHttpError(HttpError.NotFound(s"404 url: http://jsonplaceholder.typicode.com/users/${id}"))
        )
    }
  def postById(id: Int): ZIO[Any, Throwable, Response] =
    id match {
      case 1 => ZIO.readFile(getClass.getResource("postById.json").toURI.getPath).map(Response.json(_))
      case _ => ZIO.succeed(Response.fromHttpError(HttpError.NotFound(s"Post with id $id not found")))
    }

  def postsByUserId(userId: Int): ZIO[Any, Throwable, Response] =
    userId match {
      case 1 => ZIO.readFile(getClass.getResource("postByUserId.json").toURI.getPath).map(Response.json(_))
      case _ => ZIO.succeed(Response.fromHttpError(HttpError.NotFound(s"Posts with userId $userId not found")))
    }

  def getUsersBatched = ZIO.readFile(getClass.getResource("usersBatched.json").toURI.getPath).map(Response.json(_))
  def getPostsBatched = ZIO.readFile(getClass.getResource("postsBatched.json").toURI.getPath).map(Response.json(_))

  override def request(req: Request): ZIO[Any, Throwable, Response] =
    (req.url, req.method) match {
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/comments"        => comments
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/posts"           => posts
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/todos"           => todos
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/users"           => users
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/albums"          => albums
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/users/1"         => userById(1)
      case (url, Method.GET) if url == "http://jsonplaceholder.typicode.com/users/1"          => userById(1)
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/users/100"       => userById(100)
      case (url, Method.GET) if url == "http://jsonplaceholder.typicode.com/users/100"        => userById(100)
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/posts/1"         => postById(1)
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/posts?userId=1"  => postsByUserId(1)
      case (url, Method.GET)
          if url == "https://jsonplaceholder.typicode.com/users?id=5&id=2&id=1&id=10&id=3&id=8&id=7&id=6&id=9&id=4" =>
        getUsersBatched
      case (url, Method.GET)
          if url == "https://jsonplaceholder.typicode.com/posts?userId=6&userId=1&userId=5&userId=2&userId=8&userId=3&userId=9&userId=7&userId=4&userId=10" =>
        getPostsBatched
      case (url, Method.GET) if url == "https://jsonplaceholder.typicode.com/users/1/posts"   => postsByUserId(1)
      case (url, Method.POST) if url.startsWith("https://jsonplaceholder.typicode.com/users") =>
        ZIO.readFile(getClass.getResource("createUser.json").toURI.getPath).map(Response.json(_))
      case _ => ZIO.fail(new IllegalArgumentException(s"Invalid request: $req"))

    }
}
object JSONPlaceholderClient {
  def apply(): JSONPlaceholderClient = new JSONPlaceholderClient()

  def default: ULayer[JSONPlaceholderClient] = ZLayer.succeed(apply())
}

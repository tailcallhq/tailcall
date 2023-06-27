package tailcall.runtime.internal

import tailcall.runtime.http.{HttpClient, Method, Request}
import tailcall.runtime.service.FileIO
import zio.http.Response
import zio.http.model.HttpError
import zio.{ZIO, ZLayer}

import java.io.{File, FileNotFoundException}

class JSONPlaceholderClient(fileIO: FileIO) extends HttpClient {
  private def readFile(name: String)                   =
    for {
      path    <- ZIO.attempt(getClass.getResource(name).toURI.getPath).refineOrDie { case _: NullPointerException =>
        new FileNotFoundException(s"File $name not found")
      }
      file    <- ZIO.attempt(new File(path))
      content <- fileIO.read(file)
    } yield content
  val comments                                         = readFile("comments.json").map(Response.json(_))
  val posts                                            = readFile("posts.json").map(Response.json(_))
  val todos                                            = readFile("todos.json").map(Response.json(_))
  val users                                            = readFile("users.json").map(Response.json(_))
  val albums                                           = readFile("albums.json").map(Response.json(_))
  def userById(id: Int): ZIO[Any, Throwable, Response] =
    id match {
      case 1 => readFile("user-by-id.json").map(Response.json(_))
      case _ => ZIO
          .succeed(Response.fromHttpError(HttpError.NotFound(s"http://jsonplaceholder.typicode.com/users/${id}")))
    }
  def postById(id: Int): ZIO[Any, Throwable, Response] =
    id match {
      case 1 => readFile("post-by-Id.json").map(Response.json(_))
      case _ => ZIO.succeed(Response.fromHttpError(HttpError.BadRequest(s"Post with id $id not found")))
    }

  def postsByUserId(userId: Int): ZIO[Any, Throwable, Response] =
    userId match {
      case 1 => readFile("post-by-user-id.json").map(Response.json(_))
      case _ => ZIO.succeed(Response.fromHttpError(HttpError.BadRequest(s"Posts with userId $userId not found")))
    }

  def getUsersBatched = readFile("users-batched.json").map(Response.json(_))
  def getPostsBatched = readFile("posts-batched.json").map(Response.json(_))

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
        readFile("create-user.json").map(Response.json(_))
      case _ => ZIO.fail(new IllegalArgumentException(s"Invalid request: $req"))

    }
}
object JSONPlaceholderClient {
  def default: ZLayer[Any, Throwable, JSONPlaceholderClient] =
    FileIO.default >>> ZLayer.fromFunction(new JSONPlaceholderClient(_))
}

val scala2Version = "2.13.10"
val scala3Version = "3.2.2"
val zioJson       = "0.4.2"
val zioSchema     = "0.4.7"
val caliban       = "2.0.2"
val zio           = "2.0.6"
val zioHttp       = "0.0.4"

ThisBuild / scalaVersion := scala2Version

ThisBuild / scalafixDependencies += "com.github.liancheng" %% "organize-imports" % "0.6.0"

ThisBuild / scalacOptions := Seq("-language:postfixOps", "-Ywarn-unused", "-Xfatal-warnings")

ThisBuild / testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework")
ThisBuild / Test / fork       := true
Global / semanticdbEnabled    := true
Global / onChangedBuildSource := ReloadOnSourceChanges

addCommandAlias("fmt", "scalafmt; Test / scalafmt; sFix;")
addCommandAlias("fmtCheck", "scalafmtCheck; Test / scalafmtCheck; sFixCheck")
addCommandAlias("sFix", "scalafixAll; Test / scalafixAll")
addCommandAlias("sFixCheck", "scalafixAll --check; Test / scalafixAll --check")
addCommandAlias("lint", "fmt; sFix")
addCommandAlias("lintCheck", "fmtCheck; sFixCheck")
enablePlugins(JavaAppPackaging)

ThisBuild / githubWorkflowBuild ++= Seq(
  WorkflowStep.Sbt(List("Docker/stage"), name = Some("Generate Docker Files")),
  WorkflowStep.Sbt(List("lintCheck"), name = Some("Lint"), cond = Some(s"matrix.scala == '${scala2Version}'")),
  WorkflowStep.Use(
    UseRef.Public("superfly", "flyctl-actions/setup-flyctl", "master"),
    name = Some("Deploy on fly.io"),
    params = Map("api_token" -> "${{ secrets.FLY_API_TOKEN }}"),
    cond = Option("github.event_name == 'push' && github.ref == 'refs/heads/packaging-to-fly'")
  )
)
ThisBuild / githubWorkflowPublishTargetBranches := Seq()

lazy val root = (project in file(".")).aggregate(runtime, server)

lazy val runtime = (project in file("runtime")).settings(
  libraryDependencies := Seq(
    "dev.zio"               %% "zio-schema"            % zioSchema,
    "dev.zio"               %% "zio-schema-derivation" % zioSchema,
    "dev.zio"               %% "zio-schema-json"       % zioSchema,
    "com.lihaoyi"           %% "pprint"                % "0.8.1",
    "dev.zio"               %% "zio"                   % zio,
    "com.github.ghostdogpr" %% "caliban"               % caliban,
    "com.github.ghostdogpr" %% "caliban-tools"         % caliban,
    "dev.zio"               %% "zio-json"              % zioJson,
    "dev.zio"               %% "zio-json-yaml"         % zioJson,
    "dev.zio"               %% "zio-parser"            % "0.1.8",
    "dev.zio"               %% "zio-http"              % "0.0.4",

    // Testing
    "dev.zio" %% "zio-test"     % zio % Test,
    "dev.zio" %% "zio-test-sbt" % zio % Test
  )
)

Compile / mainClass := Some("tailcall.server.Main")
maintainer := "tushar@tailcall.in"

lazy val server = (project in file("server")).settings(
  libraryDependencies := Seq(
    "dev.zio" %% "zio"         % zio,
    "dev.zio" %% "zio-http"    % zioHttp,
    "dev.zio" %% "zio-rocksdb" % "0.4.2",

    // Testing
    "dev.zio" %% "zio-test"     % zio % Test,
    "dev.zio" %% "zio-test-sbt" % zio % Test
  )
).dependsOn(runtime)

// the assembly settings
// we specify the name for our fat jar
ThisBuild / assemblyMergeStrategy := { _ => MergeStrategy.first }

// removes all jar mappings in universal and appends the fat jar
Universal / mappings := {
  // universalMappings: Seq[(File,String)]
  val universalMappings = (Universal / mappings).value
  val fatJar            = (server / Compile / assembly).value
  // removing means filtering
  val filtered          = universalMappings filter { case (file, name) => !name.endsWith(".jar") }

  // add the fat jar
  filtered :+ (fatJar -> ("lib/" + fatJar.getName))
}

// the bash scripts classpath only needs the fat jar
scriptClasspath := Seq((server / assembly / assemblyJarName).value)

dockerBaseImage    := "eclipse-temurin:11"
dockerExposedPorts := Seq(8080)

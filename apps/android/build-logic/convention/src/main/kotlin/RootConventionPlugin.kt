import org.gradle.api.Plugin
import org.gradle.api.Project

class RootConventionPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        target.configureSpotless(
            kotlinTargets = listOf("build-logic/convention/src/**/*.kt"),
            gradleTargets =
                listOf(
                    "*.gradle.kts",
                    "build-logic/*.gradle.kts",
                    "build-logic/convention/*.gradle.kts",
                ),
        )
    }
}

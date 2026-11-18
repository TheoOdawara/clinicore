import com.diffplug.gradle.spotless.SpotlessExtension
import dev.detekt.gradle.extensions.DetektExtension
import org.gradle.api.GradleException
import org.gradle.api.Project
import org.gradle.api.artifacts.ProjectDependency
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.dependencies
import org.gradle.kotlin.dsl.getByType

internal val Project.libs
    get() = extensions.getByType<VersionCatalogsExtension>().named("libs")

internal fun Project.configureSpotless(kotlinTargets: List<String>, gradleTargets: List<String>) {
    pluginManager.apply("com.diffplug.spotless")
    val ktfmt = libs.findVersion("ktfmt").get().requiredVersion
    extensions.configure<SpotlessExtension> {
        kotlin {
            target(kotlinTargets)
            ktfmt(ktfmt).kotlinlangStyle()
        }
        kotlinGradle {
            target(gradleTargets)
            ktfmt(ktfmt).kotlinlangStyle()
        }
    }
}

internal fun Project.configureQuality() {
    configureSpotless(kotlinTargets = listOf("src/**/*.kt"), gradleTargets = listOf("*.gradle.kts"))

    pluginManager.apply("dev.detekt")
    extensions.configure<DetektExtension> {
        buildUponDefaultConfig.set(true)
        config.setFrom(rootProject.file("detekt.yml"))
    }
    dependencies { add("detektPlugins", libs.findLibrary("compose-rules-detekt").get()) }

    pluginManager.apply("org.jetbrains.kotlinx.kover")
    rootProject.dependencies.add("kover", rootProject.dependencies.project(path))

    dependencyLocking { lockAllConfigurations() }

    val securityFloors = libs.libraryAliases.filter { it.startsWith("security.floor.") }
    configurations.configureEach {
        if (isCanBeDeclared) {
            securityFloors.forEach { alias ->
                project.dependencies.constraints.add(name, libs.findLibrary(alias).get())
            }
        }
    }

    val from = path
    configurations.configureEach {
        dependencies.configureEach {
            if (this is ProjectDependency && path != from) {
                moduleBoundaryViolation(from, path)?.let { throw GradleException(it) }
            }
        }
    }
}

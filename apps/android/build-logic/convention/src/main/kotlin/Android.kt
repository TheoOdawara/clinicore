import com.android.build.api.dsl.CommonExtension
import org.gradle.api.Project
import org.gradle.kotlin.dsl.configure
import org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension

internal const val COMPILE_SDK = 37
internal const val MIN_SDK = 24
internal const val JDK = 21

internal fun Project.configureAndroid(android: CommonExtension) {
    android.apply {
        compileSdk = COMPILE_SDK
        defaultConfig.minSdk = MIN_SDK
        lint.apply {
            warningsAsErrors = true
            abortOnError = true
            checkReleaseBuilds = false
            lintConfig = rootProject.file("lint.xml")
        }
        testOptions.unitTests.isIncludeAndroidResources = true
    }
    extensions.configure<KotlinAndroidProjectExtension> {
        jvmToolchain(JDK)
        compilerOptions.allWarningsAsErrors.set(true)
    }
}

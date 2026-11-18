package br.com.clinicore.app

import com.lemonappdev.konsist.api.container.KoScope
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.ext.list.withNameEndingWith

internal object ArchitectureRules {
    private fun KoClassDeclaration.isViewModel() =
        name.endsWith("ViewModel") || hasParentWithName("ViewModel")

    private fun KoFileDeclaration.inFeature() = path.contains("/feature/")

    fun misplacedViewModels(scope: KoScope): List<String> =
        scope
            .classes()
            .filter { it.isViewModel() }
            .filterNot {
                it.path.contains(Regex("/feature/[^/]+/impl/")) &&
                    it.name.endsWith("ViewModel") &&
                    it.hasAnnotationWithName("HiltViewModel")
            }
            .map { it.name }

    fun androidInPureModules(scope: KoScope): List<String> =
        scope.files
            .filter { it.path.contains("/core/model/") || it.path.contains("/core/common/") }
            .filter { file -> file.imports.any { it.name.startsWith("android.") } }
            .map { it.name }

    fun dataTypesInFeatures(scope: KoScope): List<String> {
        val features = scope.files.filter { it.inFeature() }
        val classes =
            features.flatMap { it.classes() }.withNameEndingWith("Dao", "Response").map { it.name }
        val retrofit =
            features
                .filter { file -> file.imports.any { it.name.startsWith("retrofit2.") } }
                .map { it.name }
        return classes + retrofit
    }

    fun mutableUiStates(scope: KoScope): List<String> =
        scope
            .classes()
            .withNameEndingWith("UiState")
            .filter { state ->
                state.properties().any {
                    it.isVar || it.type?.name?.startsWith("Mutable") == true
                }
            }
            .map { it.name }
}

package br.com.clinicore.app

import com.google.common.truth.Truth.assertThat
import com.lemonappdev.konsist.api.Konsist
import org.junit.Test

class ArchitectureTest {
    private val fixtures = Konsist.scopeFromDirectory("app/src/test/konsist-fixtures")
    private val project = Konsist.scopeFromProject() - fixtures

    @Test
    fun `a ViewModel lives only in a feature impl, with the suffix and HiltViewModel`() {
        assertThat(ArchitectureRules.misplacedViewModels(project)).isEmpty()
        assertThat(ArchitectureRules.misplacedViewModels(fixtures))
            .containsExactly("PatientListViewModel", "PatientListPresenter", "SyncViewModel")
    }

    @Test
    fun `core model and core common never import android`() {
        assertThat(ArchitectureRules.androidInPureModules(project)).isEmpty()
        assertThat(ArchitectureRules.androidInPureModules(fixtures)).containsExactly("Patient")
    }

    @Test
    fun `a feature holds no Dao, Response or Retrofit`() {
        assertThat(ArchitectureRules.dataTypesInFeatures(project)).isEmpty()
        assertThat(ArchitectureRules.dataTypesInFeatures(fixtures))
            .containsExactly("PatientDao", "PatientResponse", "PatientDetailViewModel")
    }

    @Test
    fun `a UiState is immutable`() {
        assertThat(ArchitectureRules.mutableUiStates(project)).isEmpty()
        assertThat(ArchitectureRules.mutableUiStates(fixtures))
            .containsExactly("PatientListUiState")
    }
}

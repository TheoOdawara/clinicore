import com.google.common.truth.Truth.assertThat
import org.junit.Test

class ModuleBoundariesTest {
    private fun allows(from: String, to: String) = moduleBoundaryViolation(from, to) == null

    @Test
    fun `app joins the features and the core`() {
        assertThat(allows(":app", ":feature:patients:impl")).isTrue()
        assertThat(allows(":app", ":feature:patients:api")).isTrue()
        assertThat(allows(":app", ":core:designsystem")).isTrue()
    }

    @Test
    fun `a feature impl uses the core and the api of another feature`() {
        assertThat(allows(":feature:patients:impl", ":core:data")).isTrue()
        assertThat(allows(":feature:patients:impl", ":core:designsystem")).isTrue()
        assertThat(allows(":feature:patients:impl", ":feature:schedule:api")).isTrue()
    }

    @Test
    fun `a feature impl never reaches the data sources nor another impl`() {
        assertThat(allows(":feature:patients:impl", ":core:network")).isFalse()
        assertThat(allows(":feature:patients:impl", ":core:database")).isFalse()
        assertThat(allows(":feature:patients:impl", ":core:datastore")).isFalse()
        assertThat(allows(":feature:patients:impl", ":feature:schedule:impl")).isFalse()
    }

    @Test
    fun `a feature api depends only on the model`() {
        assertThat(allows(":feature:patients:api", ":core:model")).isTrue()
        assertThat(allows(":feature:patients:api", ":core:common")).isFalse()
        assertThat(allows(":feature:patients:api", ":feature:schedule:api")).isFalse()
    }

    @Test
    fun `core data depends on the data sources, the model and common`() {
        listOf(":core:network", ":core:database", ":core:datastore", ":core:model", ":core:common")
            .forEach { assertThat(allows(":core:data", it)).isTrue() }
        assertThat(allows(":core:data", ":core:designsystem")).isFalse()
    }

    @Test
    fun `the core never depends on a feature`() {
        assertThat(allows(":core:ui", ":core:designsystem")).isTrue()
        assertThat(allows(":core:ui", ":feature:patients:api")).isFalse()
        assertThat(allows(":core:designsystem", ":app")).isFalse()
    }

    @Test
    fun `a module outside the table is rejected with the rule that names it`() {
        assertThat(moduleBoundaryViolation(":tools", ":core:model"))
            .isEqualTo(":tools may not depend on :core:model (apps/android/AGENTS.md, Fronteiras)")
    }
}

private val dataSources = setOf(":core:network", ":core:database", ":core:datastore")
private val coreDataDependencies = dataSources + setOf(":core:model", ":core:common")

private fun String.isCore() = startsWith(":core:")

private fun String.isFeatureApi() = startsWith(":feature:") && endsWith(":api")

private fun String.isFeatureImpl() = startsWith(":feature:") && endsWith(":impl")

internal fun moduleBoundaryViolation(from: String, to: String): String? {
    val allowed =
        when {
            from == ":app" -> to.isCore() || to.isFeatureApi() || to.isFeatureImpl()
            from.isFeatureImpl() -> (to.isCore() && to !in dataSources) || to.isFeatureApi()
            from.isFeatureApi() -> to == ":core:model"
            from == ":core:data" -> to in coreDataDependencies
            from.isCore() -> to.isCore()
            else -> false
        }
    return if (allowed) null else "$from may not depend on $to (apps/android/AGENTS.md, Fronteiras)"
}

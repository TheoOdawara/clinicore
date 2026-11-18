package fixture.core.common

import java.time.Instant

fun interface Clock {
    fun now(): Instant
}

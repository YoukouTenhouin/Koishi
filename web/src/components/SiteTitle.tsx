import { FC } from "react"
import { useNavigate } from "react-router"

const SiteTitle: FC = () => {
    const navigate = useNavigate()
    return (
        <span
            style={{
                fontSize: "2em",
                fontWeight: "bold",
                cursor: "pointer"
            }}
            onClick={() => navigate("/")}
        >Koishi</span>
    )
}

export default SiteTitle
